# Phase 2 Sécurité : Coverage + Mutations + SBOM

## Contexte

Phase 1 (PR #4, mergée) : fuzzing + udeps + auditable. CI avec 10 jobs verts.

**Objectif** : 3 outils qualité supplémentaires — coverage ligne, mutation testing, SBOM CycloneDX. Aucun ne bloque le merge (informatifs). Aucun nouveau Ruleset.

---

## 1. cargo-llvm-cov — Code coverage

### Implémentation
- Job `coverage` dans `ci.yml` (ubuntu-latest, stable)
- Installation via `taiki-e/install-action@v2` (cargo-llvm-cov)
- `cargo llvm-cov --workspace --exclude rr-tauri --lcov --output-dir coverage/`
- Artifact HTML + LCOV consultable
- Pas de seuil — informatif

### Fichiers
- `.github/workflows/ci.yml` — nouveau job `coverage`

---

## 2. cargo-mutants — Mutation testing

### Implémentation
- Job `mutants` dans `ci.yml` (ubuntu-latest, stable)
- Installation via `cargo install cargo-mutants --locked`
- `cargo mutants --workspace --exclude rr-tauri` (deep/exhaustif — ~30 min)
- `timeout-minutes: 45` dans le job
- `.cargo/mutants.toml` pour exclure binaires et fuzz targets
- Artifact rapport HTML

### Fichiers
- `.github/workflows/ci.yml` — nouveau job `mutants`
- `.cargo/mutants.toml` — config d'exclusion

---

## 3. SBOM CycloneDX — Inventaire dépendances

### Implémentation
- Job `sbom` dans `ci.yml` (ubuntu-latest, stable)
- `needs: [build-cli]` — récupère le binaire produit par build-cli
- Installation via `cargo install auditable2cdx --locked`
- `auditable2cdx target/release/rr > sbom-cyclonedx.json`
- Artifact JSON

### Fichiers
- `.github/workflows/ci.yml` — nouveau job `sbom`

---

## Changements dans la CI

### Nouveaux jobs
| Job | Runner | Dépend | Durée estimée |
|-----|--------|--------|---------------|
| `coverage` | ubuntu-latest | — | ~2 min |
| `mutants` | ubuntu-latest | — | ~30 min |
| `sbom` | ubuntu-latest | build-cli | < 1 min |

### Status checks requis
Inchangé (8 existants). Ces 3 jobs sont informatifs — pas obligatoires pour merge.

---

## Fichiers modifiés
| Fichier | Changement |
|---------|------------|
| `.github/workflows/ci.yml` | +3 jobs |
| `.cargo/mutants.toml` | Nouveau |

---

## Non inclus
- Seuils de coverage : prématuré (29 tests, code jeune)
- Blocage PR par mutants : informatif seulement
- OSS-Fuzz : maintenance lourde, prématuré pre-1.0
