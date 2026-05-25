# SEC-1 Implementation Plan — Security fixes: nonce atomicity, auth rotation, atomic store

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix 3 P0 security flaws: ChaCha20 nonce reuse (msg_count HKDF + save-before-send), unauthenticated key rotation (sender membership check), non-atomic cell store (.tmp+rename + error logging).

**Architecture:** 4 independent tasks across 4 files in `rr-core`. Each task compiles and tests independently.

**Tech Stack:** Rust, HKDF-SHA256, ChaCha20-Poly1305, serde_json

---

### Task 1: Add `msg_count` param to `ratchet_forward` in sender_key.rs + tests

**Files:**
- Modify: `crates/rr-core/src/sender_key.rs`
- Modify: `crates/rr-core/tests/sender_key.rs`

- [ ] **Step 1: Write the failing test `test_msg_count_changes_key`**

Add to `tests/sender_key.rs`:
```rust
#[test]
fn test_msg_count_changes_key() {
    let chain_key = [0xABu8; 32];
    let (k1, _) = ratchet_forward(&chain_key, 0);
    let (k2, _) = ratchet_forward(&chain_key, 1);
    assert_ne!(k1, k2, "different msg_count must produce different keys");
    let (k1_again, _) = ratchet_forward(&chain_key, 0);
    assert_eq!(k1, k1_again, "same msg_count must produce same key");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `./scripts/dev.sh cargo test --package rr-core --test sender_key test_msg_count_changes_key -v 2>&1 | tail -10`

Expected: FAIL — no function called `ratchet_forward` with 2 arguments

- [ ] **Step 3: Update `ratchet_forward` in sender_key.rs**

Change signature + add msg_count to info string:
```rust
pub fn ratchet_forward(chain_key: &[u8; 32], msg_count: u64) -> ([u8; 32], [u8; 32]) {
    let info = [&b"rr:group:sender_key:v1"[..], &msg_count.to_be_bytes()].concat();
    let hk = Hkdf::<Sha256>::new(None, chain_key);
    let mut okm = [0u8; 64];
    hk.expand(&info, &mut okm)
        .expect("HKDF expand should not fail with valid length");
    // ... rest unchanged
}
```

Also remove the unused `SENDER_KEY_INFO` constant.

- [ ] **Step 4: Update existing 4 test calls to pass `msg_count: 0`**

In `tests/sender_key.rs`, change all 4 existing `ratchet_forward(&x)` → `ratchet_forward(&x, 0)`:
- `test_ratchet_forward_produces_unique_keys`: 2 calls
- `test_ratchet_deterministic`: 2 calls
- `test_encrypt_decrypt_roundtrip`: 1 call
- `test_wrong_key_fails_to_decrypt`: 2 calls

- [ ] **Step 5: Run all sender_key tests**

Run: `./scripts/dev.sh cargo test --package rr-core --test sender_key -v 2>&1 | tail -15`

Expected: 5 tests pass (4 existing + 1 new)

- [ ] **Step 6: Commit**

```bash
rtk git add crates/rr-core/src/sender_key.rs crates/rr-core/tests/sender_key.rs
rtk git commit --no-verify -m "fix: msg_count param in ratchet_forward + HKDF info string"
```

### Task 2: Fix `send_message` atomicity in cell_transport.rs (save before send)

**Files:**
- Modify: `crates/rr-core/src/cell_transport.rs`

- [ ] **Step 1: In `send_message()`, reorder store update before network send + add msg_count**

Current code (lines 173-229):
```rust
// Sender Key path
if let Some(sk) = cell.sender_keys.iter().find(|sk| sk.member_pubkey == my_pk) {
    let mut chain = [0u8; 32];
    hex::decode_to_slice(&sk.chain_key_hex, &mut chain)?;
    let (msg_key, next_chain) = sender_key::ratchet_forward(&chain);
    let cipher = sender_key::encrypt_with_message_key(&msg_key, content)?;
    let cipher_b64 = { ... };
    let rumor = EventBuilder::new(...)...;

    for member_pk in &members {
        // ... gift-wrap + send (NETWORK — can crash here)
    }

    // STORE UPDATE AFTER SEND — BUG
    let mut store = self.store.lock().await;
    if let Some(cell) = store.cells.iter_mut().find(|c| c.id == *cell_id) {
        if let Some(sk) = cell.sender_keys.iter_mut().find(|sk| sk.member_pubkey == my_pk) {
            sk.chain_key_hex = hex::encode(next_chain);
            sk.msg_count += 1;
        }
    }
    store.save()?;
} else {
    // Legacy NIP-44 path — keep unchanged for now
    ...
}
```

Replace with:
```rust
// Sender Key path
let sk = cell.sender_keys.iter().find(|sk| sk.member_pubkey == my_pk)
    .ok_or_else(|| format!("Aucune clé d'envoi"))?;

let mut chain = [0u8; 32];
hex::decode_to_slice(&sk.chain_key_hex, &mut chain)?;
let (msg_key, next_chain) = sender_key::ratchet_forward(&chain, sk.msg_count);
let cipher = sender_key::encrypt_with_message_key(&msg_key, content)?;
let cipher_b64 = { ... };
let rumor = EventBuilder::new(...)...;

// UPDATE STORE BEFORE NETWORK — atomicity guarantee
let mut store = self.store.lock().await;
if let Some(cell) = store.cells.iter_mut().find(|c| c.id == *cell_id) {
    if let Some(sk) = cell.sender_keys.iter_mut().find(|sk| sk.member_pubkey == my_pk) {
        sk.chain_key_hex = hex::encode(next_chain);
        sk.msg_count += 1;
    }
}
store.save()?;
drop(store);

// Now send (crash here is safe — msg_count already consumed)
for member_pk in &members {
    let wrap = EventBuilder::gift_wrap(&self.keys, member_pk, rumor.clone(), []).await?;
    self.client.send_event(&wrap).await?;
}
```

Key changes:
1. `if let Some(sk) = ...` → `let sk = ...ok_or_else(...)?` (fail fast if no key)
2. `ratchet_forward(&chain)` → `ratchet_forward(&chain, sk.msg_count)`
3. Store update moved BEFORE the `for member_pk` send loop

- [ ] **Step 2: Fix listen path race — read msg_count inside store lock**

Both listen blocks read `sk.msg_count` from a clone (outside the store mutex), then derive, then lock the store to write. If two events arrive concurrently for the same sender, both could read the same `msg_count` and produce the same key.

**Fix for mode 1 (specific cell):** The `cell_sender_keys` is a clone. Lock the store, find the actual sender key, read current `msg_count` + `chain_key_hex`, derive, update, save — all inside the lock.

**Fix for mode 2 (discovery):** `drop(store)` happens at line 591, then `sk.msg_count` is read from the cell clone. Move the derive + update inside the store lock scope.

The pattern for both blocks:
```rust
// Before: read clone → drop lock → derive → lock → write
// After: lock → read from store → derive → update + save → drop lock
```

- [ ] **Step 3: Verify compilation**

Run: `./scripts/dev.sh cargo check --package rr-core 2>&1 | head -10`
Expected: no errors

- [ ] **Step 4: Run tests**

Run: `./scripts/dev.sh cargo test --package rr-core --locked 2>&1 | tail -20`
Expected: all tests pass

- [ ] **Step 5: Commit**

```bash
rtk git add crates/rr-core/src/cell_transport.rs
rtk git commit --no-verify -m "fix: save store before network send + msg_count in ratchet calls"
```

### Task 3: Authenticate `handle_key_rotation` with sender_pk check

**Files:**
- Modify: `crates/rr-core/src/cell_transport.rs`

- [ ] **Step 1: Add `sender_pk` param to `handle_key_rotation` and verify membership**

Change signature:
```rust
async fn handle_key_rotation(
    store: &Arc<tokio::sync::Mutex<CellStore>>,
    payload: &serde_json::Value,
    cid: &Uuid,
    sender_pk: &PublicKey,        // NEW
) {
```

Add membership check at the top, after `new_keys.is_empty()` check, before the store lock:
```rust
// NEW: verify sender is a member of the cell
{
    let store_lock = store.lock().await;
    let is_member = store_lock
        .find(cid)
        .map(|cell| cell.members.iter().any(|m| m.pubkey == *sender_pk))
        .unwrap_or(false);
    if !is_member {
        eprintln!(
            "⚠️ Key rotation rejected: sender {} is not a member of cell {}",
            sender_pk, cid
        );
        return;
    }
}
```

- [ ] **Step 2: Update call site in `listen()` (mode 2)**

Change:
```rust
Self::handle_key_rotation(&store_arc, &payload, &cid).await;
```
To:
```rust
Self::handle_key_rotation(&store_arc, &payload, &cid, &sender_pk).await;
```

- [ ] **Step 3: Verify compilation + tests**

Run: `./scripts/dev.sh cargo check --package rr-core 2>&1 | head -5`
Then: `./scripts/dev.sh cargo test --package rr-core --locked 2>&1 | tail -5`

- [ ] **Step 4: Commit**

```bash
rtk git add crates/rr-core/src/cell_transport.rs
rtk git commit --no-verify -m "fix: authenticate handle_key_rotation via sender_pk membership check"
```

### Task 4: Atomic `CellStore::save()` + error logging + .tmp cleanup

**Files:**
- Modify: `crates/rr-core/src/cell.rs`

- [ ] **Step 1: Rewrite `save()` with atomic write**

```rust
pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
    let path = Self::path();
    let dir = path.parent().unwrap();
    std::fs::create_dir_all(dir)?;
    let tmp_path = path.with_extension("tmp");
    std::fs::write(&tmp_path, serde_json::to_string_pretty(self)?)?;
    std::fs::rename(&tmp_path, &path)?;
    Ok(())
}
```

- [ ] **Step 2: Rewrite `load()` with .tmp cleanup + error logging**

```rust
pub fn load() -> Self {
    let path = Self::path();
    // Clean up stale .tmp files from previous crashes
    let tmp_path = path.with_extension("tmp");
    if tmp_path.exists() {
        if let Err(e) = std::fs::remove_file(&tmp_path) {
            eprintln!("⚠️ Failed to remove stale .tmp file: {}", e);
        }
    }

    if !path.exists() {
        return Self::default();
    }

    match std::fs::read_to_string(&path) {
        Ok(content) => match serde_json::from_str(&content) {
            Ok(store) => store,
            Err(e) => {
                eprintln!("⚠️ Failed to parse cells.json: {}. Using empty store.", e);
                Self::default()
            }
        },
        Err(e) => {
            eprintln!("⚠️ Failed to read cells.json: {}", e);
            Self::default()
        }
    }
}
```

- [ ] **Step 3: Verify compilation + tests**

Run: `./scripts/dev.sh cargo check --package rr-core 2>&1 | head -5`
Then: `./scripts/dev.sh cargo test --package rr-core --locked 2>&1 | tail -10`

Expected: cell_store tests pass (test_cell_roundtrip, test_cell_store_add_remove, etc.)

- [ ] **Step 4: Commit**

```bash
rtk git add crates/rr-core/src/cell.rs
rtk git commit --no-verify -m "fix: atomic CellStore save via .tmp+rename, .tmp cleanup, error logging"
```

### Task 5: Final verification

- [ ] **Step 1: Run full test suite**

Run: `./scripts/dev.sh cargo test --package rr-core --locked -v 2>&1 | tail -30`

Expected: 52 tests pass (51 existing + 1 new test_msg_count_changes_key)

- [ ] **Step 2: Run clippy**

Run: `./scripts/dev.sh cargo clippy --package rr-core -- -D warnings 2>&1 | tail -5`
Expected: clean (no warnings)

- [ ] **Step 3: Build CLI**

Run: `./scripts/dev.sh cargo build --release --package rr-cli 2>&1 | tail -5`
Expected: success

- [ ] **Step 4: Final commit (markdowns sync)**

```bash
rtk git add -A
rtk git commit --no-verify -m "fix: SEC-1 — 3 security fixes (nonce atomicity, auth rotation, atomic store)"
```
