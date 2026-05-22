# Phase 1 Sécurité Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add fuzzing (crypto/identity), unused-dep detection, and binary-level auditing to CI.

**Architecture:** 3 independent tools, each adding/modifying one job in `ci.yml`. Fuzz targets live in `crates/rr-core/fuzz/`. Branch protection updated to require 2 new checks.

**Tech Stack:** cargo-fuzz (nightly, libFuzzer), cargo-udeps (nightly), cargo-auditable (stable), GitHub Actions, Rulesets API.

---
### Task 1: Add nightly toolchain and cargo-udeps job to CI

**Files:**
- Modify: `.github/workflows/ci.yml`

- [ ] **Step 1: Read current ci.yml**

```bash
cat .github/workflows/ci.yml
```

- [ ] **Step 2: Add `udeps` job after `build-cli`**

Insert this block at the end of ci.yml (after the `build-cli` job, before EOF):

```yaml
  udeps:
    name: udeps
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@nightly
      - uses: taiki-e/install-action@v2
        with:
          tool: cargo-udeps
      - run: cargo +nightly udeps --workspace --exclude rr-tauri
```

- [ ] **Step 3: Verify indentation is valid YAML**

Run: `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml')); print('valid')"`
Expected: `valid`

- [ ] **Step 4: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "ci: add cargo-udeps job (unused dependencies detection)"
```

---

### Task 2: Set up cargo-fuzz workspace and targets

**Files:**
- Create: `crates/rr-core/fuzz/Cargo.toml`
- Create: `crates/rr-core/fuzz/fuzz_targets/fuzz_nip44_roundtrip.rs`
- Create: `crates/rr-core/fuzz/fuzz_targets/fuzz_nip44_decrypt.rs`
- Create: `crates/rr-core/fuzz/fuzz_targets/fuzz_identity_parse.rs`
- Modify: `Cargo.toml` (root, add workspace member)

- [ ] **Step 1: Create fuzz directory structure**

```bash
mkdir -p crates/rr-core/fuzz/fuzz_targets
```

- [ ] **Step 2: Create fuzz/Cargo.toml**

Write `crates/rr-core/fuzz/Cargo.toml`:

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

[dependencies.rr-core]
path = ".."

# Prevent this from interfering with workspaces
[workspace]
```

- [ ] **Step 3: Create fuzz target — NIP-44 roundtrip**

Write `crates/rr-core/fuzz/fuzz_targets/fuzz_nip44_roundtrip.rs`:

```rust
#![no_main]

use libfuzzer_sys::fuzz_target;
use rr_core::crypto::nip44;

fuzz_target!(|data: &[u8]| {
    // data layout: [32 bytes secret key] [payload]
    // payload max 65535 bytes (NIP-44 limit)
    if data.len() < 32 {
        return;
    }
    let payload = &data[32..];
    if payload.len() > 65535 {
        return;
    }
    let Ok(sk) = nostr::SecretKey::from_slice(&data[..32]) else { return };
    let pk = nostr::PublicKey::from(&sk);
    let Ok(conv_key) = nostr::nips::nip44::ConversationKey::new(&sk, &pk) else { return };
    let conv_key_clone = conv_key.clone();
    if let Ok(ciphertext) = nip44::encrypt(payload, &conv_key) {
        if let Ok(plaintext) = nip44::decrypt(&ciphertext, &conv_key_clone) {
            assert_eq!(plaintext, payload);
        }
    }
});
```

- [ ] **Step 4: Create fuzz target — NIP-44 decrypt with invalid ciphertexts**

Write `crates/rr-core/fuzz/fuzz_targets/fuzz_nip44_decrypt.rs`:

```rust
#![no_main]

use libfuzzer_sys::fuzz_target;
use rr_core::crypto::nip44;

fuzz_target!(|data: &[u8]| {
    if data.len() < 32 {
        return;
    }
    let Ok(sk) = nostr::SecretKey::from_slice(&data[..32]) else { return };
    let pk = nostr::PublicKey::from(&sk);
    let Ok(conv_key) = nostr::nips::nip44::ConversationKey::new(&sk, &pk) else { return };
    // ciphertext is rest of input (may be empty, invalid, truncated, etc.)
    let ciphertext = &data[32..];
    // Should never panic on any input — only return Err
    let _ = nip44::decrypt(ciphertext, &conv_key);
});
```

- [ ] **Step 5: Create fuzz target — identity parsing**

Write `crates/rr-core/fuzz/fuzz_targets/fuzz_identity_parse.rs`:

```rust
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // Try parsing as nsec/npub bech32 — should never panic, only Err
        let _ = s.parse::<nostr::Keys>();
        // Try hex or mnemonic — should never panic, only Err
        let _ = nostr::Keys::parse(s);
    }
});
```

**Important:** The fuzz directory has its own `[workspace]` in Cargo.toml — do NOT add it to root workspace members. cargo-fuzz handles standalone fuzzing workspaces natively.

- [ ] **Step 6: Verify fuzz targets compile**

```bash
./scripts/dev.sh bash -c "cargo +nightly check --manifest-path crates/rr-core/fuzz/Cargo.toml"
```
Expected: Compiles without errors

- [ ] **Step 7: Commit**

```bash
git add crates/rr-core/fuzz/
git commit -m "feat: add cargo-fuzz targets for NIP-44 + identity parsing"
```

---

### Task 3: Add fuzz CI job

**Files:**
- Modify: `.github/workflows/ci.yml`

- [ ] **Step 1: Read current ci.yml**

```bash
cat .github/workflows/ci.yml
```

- [ ] **Step 2: Add `fuzz` job after `udeps`**

Insert this block after the `udeps` job:

```yaml
  fuzz:
    name: fuzz
    runs-on: ubuntu-latest
    strategy:
      matrix:
        target:
          - fuzz_nip44_roundtrip
          - fuzz_nip44_decrypt
          - fuzz_identity_parse
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@nightly
      - uses: taiki-e/install-action@v2
        with:
          tool: cargo-fuzz
      - name: Restore corpus cache
        uses: actions/cache@v4
        with:
          path: crates/rr-core/fuzz/corpus/${{ matrix.target }}
          key: fuzz-corpus-${{ matrix.target }}-${{ github.sha }}
          restore-keys: |
            fuzz-corpus-${{ matrix.target }}-
      - run: cargo fuzz run ${{ matrix.target }} -- -max_total_time=120
        working-directory: crates/rr-core/fuzz
      - uses: actions/upload-artifact@v4
        if: failure()
        with:
          name: fuzz-artifacts-${{ matrix.target }}-${{ github.sha }}
          path: crates/rr-core/fuzz/artifacts/
```

Note: The `working-directory: crates/rr-core` is where fuzz/ lives. The `cargo fuzz` command looks for `./fuzz/` relative to the working directory.

- [ ] **Step 3: Verify YAML**

Run: `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml')); print('valid')"`
Expected: `valid`

- [ ] **Step 4: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "ci: add fuzz job (NIP-44 + identity, 3 targets, 2min each)"
```

---

### Task 4: Add cargo auditable to build-cli job

**Files:**
- Modify: `.github/workflows/ci.yml`

- [ ] **Step 1: Modify build-cli job to use cargo auditable**

Replace the relevant lines in the `build-cli` job:

Old:
```yaml
      - uses: swatinem/rust-cache@v2
      - run: cargo build --package rr-cli --release --locked
```

New:
```yaml
      - uses: swatinem/rust-cache@v2
      - run: cargo install cargo-auditable --locked
      - run: cargo auditable build --package rr-cli --release --locked
```

- [ ] **Step 2: Verify YAML**

Run: `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml')); print('valid')"`
Expected: `valid`

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "ci: use cargo auditable for build-cli (embed dep metadata in binary)"
```

---

### Task 5: Update GitHub Ruleset with new check names

**Files:**
- Modify: Ruleset via GitHub API (no file change)

- [ ] **Step 1: Add fuzz and udeps to Check Main ruleset**

```bash
gh api repos/giak/reseau-racine/rulesets/16747604 -X PUT --input - <<'EOF'
{
  "name": "Check Main",
  "target": "branch",
  "enforcement": "active",
  "conditions": {
    "ref_name": {
      "include": ["~DEFAULT_BRANCH"],
      "exclude": []
    }
  },
  "rules": [
    {
      "type": "required_status_checks",
      "parameters": {
        "required_status_checks": [
          {"context": "lint"},
          {"context": "test"},
          {"context": "audit"},
          {"context": "fuzz"},
          {"context": "udeps"},
          {"context": "check-cross (macos-latest)"},
          {"context": "check-cross (windows-latest)"},
          {"context": "build-cli"}
        ],
        "strict_required_status_checks_policy": false
      }
    },
    {
      "type": "pull_request",
      "parameters": {
        "dismiss_stale_reviews_on_push": false,
        "require_code_owner_review": false,
        "require_last_push_approval": false,
        "required_approving_review_count": 1,
        "required_review_thread_resolution": false
      }
    }
  ],
  "bypass_actors": [
    {
      "actor_id": 5073444,
      "actor_type": "User",
      "bypass_mode": "always"
    }
  ]
}
EOF
```

- [ ] **Step 2: Verify ruleset updated**

```bash
gh api repos/giak/reseau-racine/rulesets/16747604 --jq '.rules[0].parameters.required_status_checks[].context'
```
Expected: 8 contexts including `fuzz` and `udeps`

- [ ] **Step 3: Commit infrastucture change doc**

No file change needed. The ruleset is managed via API. Optionally update AGENTS.md if the check list changes.

---

### Task 6: Verify everything in CI

- [ ] **Step 1: Push all commits to PR branch**

```bash
git push origin feature/security-phase1
```

- [ ] **Step 2: Create PR**

```bash
gh pr create --fill
```

- [ ] **Step 3: Wait for CI to pass**

```bash
gh run list --branch feature/security-phase1 --limit 1 --json status,conclusion
```

Expected: All 8 checks green (lint, test, audit, fuzz, udeps, check-cross ×2, build-cli)

- [ ] **Step 4: Squash merge**

```bash
gh pr merge --squash --admin
```

---

### Task 7: Update documentation

**Files:**
- Modify: `docs/TRACKING.md`

- [ ] **Step 1: Add Phase 1 Securite items to EPIC 0**

Add to EPIC 0 table:
```markdown
| Fuzzing (NIP-44 roundtrip + decrypt invalid + identity parse) | ✅ |
| cargo-udeps (unused dependencies CI) | ✅ |
| cargo auditable (binary-level audit) | ✅ |
```

- [ ] **Step 2: Commit**

```bash
git add docs/TRACKING.md
git commit -m "doc: track Phase 1 sécurité (fuzz + udeps + auditable)"
```

---

## Implementation Notes (post-PR #4)

### Corrections apportées au plan initial

#### 1. fuzz/Cargo.toml : [[bin]] sections requises
Le plan initial omettait les entrées `[[bin]]`. cargo-fuzz (v0.13.x) nécessite des entrées `[[bin]]` explicites pointant vers chaque fuzz target, comme généré par `cargo fuzz init` :
```toml
[[bin]]
name = "fuzz_nip44_roundtrip"
path = "fuzz_targets/fuzz_nip44_roundtrip.rs"
test = false
doc = false
bench = false
```

#### 2. Dépendance `nostr` directe requise
Les fuzz targets utilisent `nostr::nips::nip44::{encrypt, decrypt}` et `nostr::SecretKey::from_slice` directement (pas via rr_core). Ajouter :
```toml
[dependencies]
nostr = { version = "0.44", features = ["nip44"] }
```

#### 3. API NIP-44 différente
Le plan utilisait `nip44::encrypt(payload, &conv_key)` (bas niveau, par ConversationKey).
L'API réelle (`nostr 0.44.x`) est :
```rust
// Haut niveau (base64) — ce qu'on utilise dans les fuzz targets
nip44::encrypt(&sk, &pk, payload, Version::V2)  // → Result<String, Error>
nip44::decrypt(&sk, &pk, &ciphertext)            // → Result<String, Error>

// Conversion SecretKey → PublicKey via secp256k1
let secp = secp256k1::Secp256k1::new();
let pk_secp = secp256k1::PublicKey::from_secret_key(&secp, &secp_sk);
let (xonly, _) = pk_secp.x_only_public_key();
let pk = nostr::PublicKey::from(xonly);
```
Disponible via `nostr::secp256k1` (nostr re-exporte `pub extern crate secp256k1`).

#### 4. fuzz/.gitignore nécessaire
```gitignore
target/
```
Sans ça, `git add crates/rr-core/fuzz/` capture les binaires de build (20+ MB chacun).

### CI Troubleshooting

#### Problème : musl/ASAN incompatibility
cargo-fuzz précompilé par `taiki-e/install-action@v2` détecte `x86_64-unknown-linux-musl` comme host.
AddressSanitizer incompatible avec musl statique.

**Solution :** `--target $(rustc --print host-tuple)` (issue cargo-fuzz #398)

**Tentatives échouées :**
1. `cargo +nightly install cargo-fuzz --locked` — `rustix` utilise attributes nightly-only
2. `RUSTFLAGS=-Ctarget-feature=-crt-static` — overridé par cargo-fuzz
3. `rustup target add x86_64-unknown-linux-musl` — ASAN toujours incompatible

#### working-directory
Le plan utilisait `working-directory: crates/rr-core/fuzz`. La commande correcte est :
```yaml
- run: cargo +nightly fuzz run --target $(rustc --print host-tuple) ${{ matrix.target }} -- -max_total_time=120
  working-directory: crates/rr-core
```
cargo-fuzz cherche `./fuzz/` relatif au working directory.

### Résultat
- PR #4 mergée, tous les 10 jobs CI verts
- 8 status checks dans Ruleset Check Main
- Tests : 29 pass (inchangé)
- Le fuzzer tourne 2 min par target avec corpus cache
