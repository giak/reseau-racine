# Phase 1 Sécurité : Fuzzing + udeps + auditable

## Contexte

EPIC 0 livré. CI actuelle : lint, test, audit (cargo-deny), check-cross, build-cli. 0 unsafe, 0 unwrap prod.

Objectif : ajouter 3 outils sécurité standard 2026 pour couvrir les angles morts.

---

## 1. cargo-fuzz — NIP-44 + identity

### Cibles
- `fuzz_nip44_roundtrip` : encrypt → decrypt avec payloads aléatoires (vides, 1B, max 65535B, padding bruité)
- `fuzz_nip44_decrypt` : decrypt avec ciphertexts invalides (tronqués, corrompus, clé wrong)
- `fuzz_identity_parse` : parsing nsec/npub/mnemonic invalides

### CI
- Job `fuzz` dans `ci.yml`, nightly toolchain
- `cargo fuzz run <target> -- -max_total_time=120` (2 min / target)
- Cache GitHub pour le corpus (prefix `fuzz-corpus-`)
- `if: failure()` → upload artifacts pour post-mortem

### Fichiers
- `crates/rr-core/fuzz/` — targets + corpus
- `crates/rr-core/fuzz/Cargo.toml` — workspace member
- `.github/workflows/ci.yml` — nouveau job `fuzz`

---

## 2. cargo-udeps — dépendances mortes

### Implémentation
- Nouveau job `udeps` dans `ci.yml`
- `cargo +nightly install cargo-udeps --locked`
- `cargo +nightly udeps --workspace --exclude rr-tauri`
- nightly toolchain (cargo-udeps require nightly)

### Fichiers
- `.github/workflows/ci.yml` — nouveau job `udeps`

---

## 3. cargo auditable — audit au niveau binaire

### Implémentation
- Ajouter `cargo auditable` dans le job `build-cli`
- `cargo auditable build --package rr-cli --release --locked` (remplace `cargo build`)
- Optionnel : `cargo audit` sur le binaire dans une étape séparée

### Note
`cargo auditable` remplace `cargo build` dans `build-cli`. L'artefact produit contient la metadata des dépendances dans une section ELF dédiée. Utilisable plus tard par `cargo audit binary`.

### Fichiers
- `.github/workflows/ci.yml` — modifier job `build-cli`

---

## Changements dans la CI

### Nouveaux jobs
| Job | Runner | Durée |
|-----|--------|-------|
| `fuzz` | ubuntu-latest | ~6 min (3 targets × 2 min) |
| `udeps` | ubuntu-latest | ~2 min |

### Job modifié
| Job | Changement |
|-----|------------|
| `build-cli` | `cargo build` → `cargo auditable build` |

### Status checks (Ruleset Check Main)
Ajouter `fuzz` et `udeps` aux required status checks. `build-cli` déjà présent, pas de changement de nom.

---

## Non inclus (Phase 2+)

- Coverage (tarpaulin/llvm-cov) : utile mais pas prioritaire vu 29 tests déjà passants
- SBOM cyclonedx : pertinent au moment du release
- Miri : pas de unsafe à checker
- OSS-Fuzz : maintenance lourde, prématuré en pre-1.0
