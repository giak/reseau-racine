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

### Fichiers
- `crates/rr-core/fuzz/` — targets + corpus + `.gitignore`
- `crates/rr-core/fuzz/Cargo.toml` — **standalone workspace** (ne PAS ajouter à `workspace.members` dans `Cargo.toml` root)
- `crates/rr-core/fuzz/fuzz_targets/*.rs` — 3 fuzz targets
- `.github/workflows/ci.yml` — nouveau job `fuzz`

### Structure du fuzz Cargo.toml

```toml
[package]
name = "rr-core-fuzz"
version = "0.1.0"
edition = "2021"
publish = false

[package.metadata]
cargo-fuzz = true

[dependencies]
libfuzzer-sys = "0.4"
nostr = { version = "0.44", features = ["nip44"] }

[dependencies.rr-core]
path = ".."

# Chaque target nécessite une entrée [[bin]] explicite
[[bin]]
name = "fuzz_nip44_roundtrip"
path = "fuzz_targets/fuzz_nip44_roundtrip.rs"
test = false
doc = false
bench = false

[[bin]]
name = "fuzz_nip44_decrypt"
path = "fuzz_targets/fuzz_nip44_decrypt.rs"
test = false
doc = false
bench = false

[[bin]]
name = "fuzz_identity_parse"
path = "fuzz_targets/fuzz_identity_parse.rs"
test = false
doc = false
bench = false

[workspace]
```

### API NIP-44 utilisée
Les fuzz targets utilisent `nostr::nips::nip44::{encrypt, decrypt}` (haut niveau, base64) et non les fonctions bas niveau `v2::encrypt_to_bytes`. La conversion `SecretKey → PublicKey` passe par `secp256k1::PublicKey::from_secret_key` + `x_only_public_key()`, disponible via `nostr::secp256k1`.

### CI
- Job `fuzz` dans `ci.yml` avec matrix 3 targets parallélisés
- nightly toolchain via `dtolnay/rust-toolchain@nightly`
- Installation : `taiki-e/install-action@v2` avec `cargo-fuzz`
- **Commande :** `cargo +nightly fuzz run --target $(rustc --print host-tuple) <target> -- -max_total_time=120`
- Cache GitHub Actions pour le corpus (prefix `fuzz-corpus-`, restore sans sha)
- `if: failure()` → upload artifacts fuzz via `actions/upload-artifact@v4`

#### CI Troubleshooting — musl/ASAN incompatibility

**Problème :** `taiki-e/install-action@v2` livre un binaire cargo-fuzz compilé statiquement avec musl.
cargo-fuzz détecte alors le host comme `x86_64-unknown-linux-musl` (même sur ubuntu-latest).
L'AddressSanitizer est incompatible avec musl statique :
```
error: sanitizer is incompatible with statically linked libc, disable it using `-C target-feature=-crt-static`
```

**Solution :** `--target $(rustc --print host-tuple)` force le target GNU natif (`x86_64-unknown-linux-gnu`),
compatible ASAN. Issue cargo-fuzz #398, confirmé par le projet coreutils.

**Tentatives échouées avant la solution :**
1. `cargo +nightly install cargo-fuzz --locked` — fail car cargo-fuzz dépend de `rustix` qui utilise des attributes nightly-only
2. `RUSTFLAGS=-Ctarget-feature=-crt-static` — ignoré car cargo-fuzz override RUSTFLAGS en ligne de commande
3. `rustup target add x86_64-unknown-linux-musl` — ne résout pas l'incompatibilité ASAN/musl

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
Ajouter `fuzz` et `udeps` aux required status checks. Total : 8 checks :
`lint`, `test`, `audit`, `fuzz`, `udeps`, `check-cross (macos-latest)`, `check-cross (windows-latest)`, `build-cli`

---

## Bilan Phase 1 (PR #4)

- **PR #4** mergée le 2026-05-22, 4 commits squashed
- CI 10 jobs → tous ✅ (première tentative échec musl, fixé au 2ème push)
- Tests : 29 pass (inchangé, fuzz n'ajoute pas de tests Rust)
- 9 fichiers changés (fuzz/*, ci.yml, AGENTS.md, TRACKING.md)

## Non inclus (Phase 2+)

- Coverage (tarpaulin/llvm-cov) : utile mais pas prioritaire vu 29 tests déjà passants
- SBOM cyclonedx : sera trivial via `auditable2cdx` une fois cargo-auditable en place
- Miri : pas de unsafe à checker
- OSS-Fuzz : maintenance lourde, prématuré en pre-1.0
- AFL++ / Honggfuzz : corpus compatible avec libFuzzer, pourra compléter en Phase 2
