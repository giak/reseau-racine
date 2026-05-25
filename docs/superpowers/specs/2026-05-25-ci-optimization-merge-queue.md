# CI Optimization: Merge Queue + Tiered Workflows

**Date:** 2026-05-25
**Author:** giak (after design discussion)
**Status:** Design approved, ready for implementation

## Problem

Notre CI actuelle lance 8 jobs sur **chaque** PR, incluant check-cross (macOS, Windows), audit, bench, udeps. Pour les PR docs-only (~50% de nos PRs), ces 20+ minutes sont du gaspillage total — aucun code Rust n'a changé, aucun bug ne peut être détecté.

**Métriques avant optimisation :**
- PR code : ~20-30 min wall time
- PR docs-only : ~20-30 min wall time (gaspillage total)
- Runs redondants sur force-push (pas de cancel-in-progress)

## Solution: Merge Queue + CI à 2 Tiers

Sépare le CI en 2 paliers. Le palier 1 (rapide) court sur chaque push PR. Le palier 2 (complet) ne court qu'au moment du merge, via la GitHub Merge Queue.

### Architecture

```
PR push → Tier 1: lint + test + build-cli (~2-5 min)
         → Stub Tier 2: jobs vides (~10s)
         → 8 required checks satisfaits ✅

Add to Queue → Merge queue crée gh-readonly-queue/main/...
             → Tier 1 re-run sur le merge
             → Tier 2: check-cross, audit, udeps, bench, fuzz, coverage (~8-15 min)
             → Merge auto si tout vert ✅
```

### Workflows

#### 1. `ci.yml` — Tier 1 (PR + merge_group)

**Trigger:** `pull_request`, `merge_group`

```yaml
concurrency:
  group: ${{ github.workflow }}-${{ github.head_ref || github.run_id }}
  cancel-in-progress: true
```

**Jobs:**

| Job | Runner | Commande | Condition |
|-----|--------|----------|-----------|
| `changes` | ubuntu | dorny/paths-filter | toujours |
| `lint` | ubuntu | fmt + clippy | si Rust changé |
| `test` | ubuntu | cargo test --workspace --locked | si Rust changé |
| `build-cli` | ubuntu | cargo auditable build --release --package rr-cli | si Rust changé |

**Path filter (rust):**
```yaml
rust:
  - 'crates/**/*.rs'
  - 'crates/**/Cargo.toml'
  - 'Cargo.toml'
  - 'Cargo.lock'
  - 'rust-toolchain.toml'
  - '.github/workflows/ci.yml'
  - '.github/workflows/ci-full.yml'
```

Si `changes.outputs.rust != 'true'` :
- lint → skip (needed pour fmt sur markdown ? non, seulement Rust)
- test → skip
- build-cli → skip
- Résultat : workflow se termine en ~30s (checkout + filter)

#### 2. `ci-full.yml` — Tier 2 (merge_group seulement)

**Trigger:** `merge_group` seulement

**Jobs (required = doit passer pour merge, optional = info supplémentaire) :**

| Job | Runner | Commande | Timeout | Required |
|-----|--------|----------|---------|----------|
| `changes` | ubuntu | dorny/paths-filter | 1 min | non (interne) |
| `check-cross (macos-latest)` | macOS | cargo check --workspace --locked | 15 min | **oui** |
| `check-cross (windows-latest)` | Windows | cargo check --workspace --locked | 15 min | **oui** |
| `audit` | ubuntu | cargo-deny check advisories bans licenses sources | 5 min | **oui** |
| `udeps` | ubuntu (nightly) | cargo +nightly udeps | 5 min | **oui** |
| `fuzz (fuzz_nip44_roundtrip)` | ubuntu (nightly) | cargo +nightly fuzz run target -- -max_total_time=120 | 5 min | **oui** |
| `fuzz (fuzz_nip44_decrypt)` | ubuntu (nightly) | idem | 5 min | **oui** |
| `fuzz (fuzz_identity_parse)` | ubuntu (nightly) | idem | 5 min | **oui** |
| `bench` | ubuntu | cargo bench --bench crypto | 5 min | non (info) |
| `mutants` | ubuntu | cargo mutants | 45 min | non (info) |
| `coverage` | ubuntu | cargo llvm-cov | 5 min | non (info) |
| `sbom` | ubuntu | auditable2cdx (after build-cli) | 2 min | non (info) |

**Path filtering** : sur `ci.yml` (PR), si docs-only, `lint`, `test`, `build-cli` skip. Sur `ci-full.yml` (merge_group), **pas de path filtering sur les jobs requis** — ils doivent toujours s'exécuter et reporter pour que la queue progresse. Seuls les jobs optionnels (`bench`, `mutants`, `coverage`, `sbom`) peuvent skip si docs-only.

#### 3. `ci-stub.yml` — Stub pour PR-time (pull_request_target)

**Trigger:** `pull_request_target`
**Permissions:** `{}` (zéro risques)

Émet des jobs vides dont les **noms exacts** correspondent aux jobs de ci-full.yml :

| Job name (doit matcher exactement les noms dans ci-full.yml) |
|-------------------------------------------------------------|
| `check-cross (macos-latest)` |
| `check-cross (windows-latest)` |
| `audit` |
| `udeps` |
| `fuzz (fuzz_nip44_roundtrip)` |
| `fuzz (fuzz_nip44_decrypt)` |
| `fuzz (fuzz_identity_parse)` |

Note : `bench`, `mutants`, `coverage`, `sbom` ne sont PAS des checks requis — ils n'ont pas besoin de stub. Ils tournent uniquement sur merge_group pour information.

Chaque job fait juste `run: echo "stub — real check runs on merge_group"`.

### Changements rulesets

**Check Main :**
- `on:` events : `pull_request` ✅ (inchangé)
- Required checks : les 8 noms inchangés (stubs les satisfont sur PR)
- `bypass_mode` : `giak` peut bypass (inchangé)

**Protect Main :** inchangé.

**Nouveau :** Activer **Require merge queue** dans les règles de protection de `main` :
- Merge method : squash
- Build concurrency : 5
- Only merge non-failing PRs : true
- Status check timeout : 60 min

### Changements workflow dev

#### Nouveau flux normal

```bash
# Créer PR (inchangé)
rtk git checkout -b feature/<scope>-<description>
# ... travail ...
rtk git add <files>
rtk git commit -m "<message>"
rtk git push -u origin <branch>
rtk gh pr create --fill

# CI Tier 1 court (~2-5 min). 8 checks verts (3 réels + 5 stubs).
# Quand prêt à merge : "Merge when ready" dans l'UI GitHub
# Ou via CLI :
rtk gh pr merge --squash --auto  # Add to queue, auto-merge quand CI vert

# CI Tier 2 court (~8-15 min). Merge auto si tout vert.
```

#### Flux hotfix urgent

```bash
# Option 1 : bypass ruleset (giak)
rtk gh pr merge --squash --admin   # Bypass Check Main ruleset

# Option 2 : push direct sur main
rtk git push origin main            # (si le ruleset le permet avec bypass)
```

### Bénéfices attendus

| Métrique | Avant | Après | Gain |
|----------|-------|-------|------|
| PR code feedback time | 20-30 min | 2-5 min | 4-6x |
| PR docs-only time | 20-30 min | ~30s | 40-60x |
| CI minutes/mois (estimation) | ~500 min | ~200 min | 60% |
| Force-push waste | oui | cancel-in-progress | éliminé |
| Rebase manuel | oui | merge queue gère | éliminé |

### Risques et mitigations

| Risque | Mitigation |
|--------|------------|
| Stub job name mismatch → merge queue bloqué | CODEOWNERS sur `.github/workflows/` pour forcer review des noms de jobs |
| merge_group event mal configuré → queue jamais verte | Test avec une PR canari avant d'activer sur toutes les PRs |
| merge queue timeout (60min) | `mutants` a timeout 45min, les autres <15min. safe. |
| Perte du contrôle du moment de merge | `--admin` bypass conservé pour hotfixes |
| Stub permission `pull_request_target` | `permissions: {}` explicite, pas de checkout, pas de secrets |

### Critères de succès

- [ ] PR docs-only : terminé en <1 min, 8 checks verts
- [ ] PR code : feedback Tier 1 en <5 min
- [ ] Merge queue : PR merge auto si Tier 2 vert
- [ ] Hotfix bypass : `--admin` fonctionne encore
- [ ] Cancel-in-progress : force-push annule le run précédent
- [ ] Tous les anciens checks (lint, test, audit, check-cross, build-cli, bench, udeps) toujours présents dans le ruleset

### Dépendances

- `dorny/paths-filter@v3` (déjà utilisé par la communauté, action vérifiée)
- GitHub Merge Queue (feature native GitHub, disponible sur notre plan)
