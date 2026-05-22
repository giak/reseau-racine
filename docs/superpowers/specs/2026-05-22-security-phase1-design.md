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
- Job `fuzz` dans `ci.yml`
- nightly toolchain via `dtolnay/rust-toolchain@nightly`
- Installation : `taiki-e/install-action@v2` avec `cargo-fuzz`
- `cargo fuzz run <target> -- -max_total_time=120` (2 min / target)
- Cache GitHub Actions pour le corpus (prefix `fuzz-corpus-`)
- `if: failure()` → upload artifacts via `actions/upload-artifact@v4`

### Fichiers
- `crates/rr-core/fuzz/` — targets + corpus
- `crates/rr-core/fuzz/Cargo.toml` — workspace member (ajouter à `workspace.members` dans `Cargo.toml` root)
- `.github/workflows/ci.yml` — nouveau job `fuzz`

---

## 2. cargo-udeps — dépendances mortes

### Implémentation
- Nouveau job `udeps` dans `ci.yml`
- nightly toolchain via `dtolnay/rust-toolchain@nightly`
- Installation : `taiki-e/install-action@v2` avec `cargo-udeps`
- `cargo +nightly udeps --workspace --exclude rr-tauri`

### Fichiers
- `.github/workflows/ci.yml` — nouveau job `udeps`

---

## 3. cargo auditable — audit au niveau binaire

### Implémentation
- Ajouter `cargo install cargo-auditable --locked` dans le job `build-cli` (avant `cargo auditable build`)
- Remplacer `cargo build` par `cargo auditable build --package rr-cli --release --locked`
- Optionnel : `cargo audit binary ./target/release/rr` dans une étape séparée (nécessite `cargo install cargo-audit`)

### Note
Le binaire produit contient la metadata des dépendances dans une section ELF dédiée. Utilisable plus tard par `cargo audit binary` sans avoir à re-scanner le workspace.

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
