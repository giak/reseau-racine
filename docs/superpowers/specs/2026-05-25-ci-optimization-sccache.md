# CI Optimization: sccache Layer

**Date:** 2026-05-25
**Author:** giak (after design discussion)
**Status:** Design approved

## Problem

Le CI actuel utilise `Swatinem/rust-cache@v2` pour le cache au niveau crate — restauration de `target/` entre les runs. Mais chaque run a 5+ jobs Rust qui compilent séquentiellement : le job `lint` compile, puis `test` recompile, etc. Le cache crate ne partage pas les objets entre jobs du même run.

Avec path filtering déjà en place, les runs PR docs-only sont ~30s. Les runs PR code restent à 5-15 min. Le bottleneck est la compilation séquentielle.

## Solution : sccache layer

Ajouter `mozilla-actions/sccache-action` à chaque job Rust, avec `RUSTC_WRAPPER=sccache` et `SCCACHE_GHA_ENABLED=true`. Les artefacts objets sont stockés dans le cache GHA et partagés entre tous les jobs d'un même run et entre les runs.

sccache est complémentaire à `Swatinem/rust-cache` :
- `Swatinem/rust-cache` : cache le `target/` entier entre les runs (restore/save)
- sccache : cache les fichiers `.o` individuels, partagés entre jobs ET entre runs
- Ensemble : premier job compile, les suivants prennent les objets déjà compilés

### Changements

**`ci.yml`** — ajouter dans chaque job Rust (sauf fuzz/udeps/mutants/sbom) :

```yaml
- uses: mozilla-actions/sccache-action@v0
- env:
    RUSTC_WRAPPER: sccache
    SCCACHE_GHA_ENABLED: "true"
```

### Jobs concernés (8)

| Job | Raison |
|-----|--------|
| lint | rustfmt + clippy → compile |
| test | cargo test → compile |
| audit | cargo deny (vérifie les dépendances) |
| check-cross (macos) | cargo check |
| check-cross (windows) | cargo check |
| build-cli | cargo auditable build --release |
| bench | cargo bench |
| coverage | cargo llvm-cov |

**Exclus :**
- `changes` (pas de compilation)
- `fuzz` (cargo-fuzz gère son propre cache)
- `udeps` (nightly, minimal compilation)
- `mutants` (timeout 45min déjà, compilation non bottleneck)
- `sbom` (dépend de l'artefact build-cli, pas de compilation propre)
- `release` (même template, mais tag-driven)

### Bénéfices attendus

| Métrique | Avant sccache | Après sccache | Gain estimé |
|----------|---------------|---------------|-------------|
| PR code — jobs après le 1er (même run) | 5-15 min cumulé | jobs 2+ prennent les objets déjà compilés | 2-3x sur le total |
| PR code — runs suivants cache chaud | 5-15 min | ~2-5 min | 2-3x |
| PR docs-only | ~30s | ~30s | inchangé (déjà skip) |

Le gain principal est sur les runs séquentiels (lint → test → audit → ...) où sccache fournit les objets déjà compilés par le job précédent dans le même run.

### Risques et mitigations

| Risque | Mitigation |
|--------|------------|
| sccache store dans le cache GHA → consomme du quota | Cache GHA 10GB sur free plan. sccache + rust-cache < 2GB pour ce workspace |
| sccache cache miss → overhead de ~2s par job | Négligeable vs temps de compilation |
| sccache bug → compilation échoue | sccache a `--sccache-stop` fallback automatique |

### Critères de succès

- [ ] PR code : jobs séquentiels dans le même run 2x plus rapides (sccache cache les .o entre jobs)
- [ ] Pas de régression : tous les jobs Rust passent avec sccache
- [ ] Pas de warning/miss dans les logs sccache
