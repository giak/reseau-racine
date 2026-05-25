# CLEAN-1 Implementation Plan — Dead Code Removal

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove 4 dead code artifacts: `CryptoProvider` wrapper, `MessageService` struct, `TransportProvider` trait, legacy NIP-44 path in listen.

**Architecture:** 4 independent tasks across 6 source files + 2 test files. Each compiles independently. Run order: MessageService → TransportProvider → Legacy path → CryptoProvider.

**Tech Stack:** Rust, nostr-rs (NIP-44)

**Pre-flight check:** Each task starts with `cargo check --package rr-core` to verify before/after. Final verification runs full suite.

---

### Task 1: MessageService struct → free functions

**Files:**
- Modify: `crates/rr-core/src/message.rs`
- Modify: `crates/rr-core/src/lib.rs`
- Modify: `crates/rr-cli/src/main.rs`

- [ ] **Step 1: Convert struct to free functions in message.rs**

Replace:
```rust
use nostr::prelude::*;
use nostr_sdk::prelude::*;

#[derive(Debug, Clone)]
pub struct MessageService;

impl MessageService {
    pub fn new() -> Self {
        Self
    }

    pub async fn send(
        &self,
        client: &Client,
        receiver_pubkey: PublicKey,
        content: &str,
    ) -> Result<EventId, Box<dyn std::error::Error>> {
        let output = client
            .send_private_msg(receiver_pubkey, content, vec![])
            .await?;
        if output.success.is_empty() {
            let errors: Vec<String> = output
                .failed
                .iter()
                .map(|(url, err)| format!("{url}: {err}"))
                .collect();
            return Err(format!("Échec d'envoi: {}", errors.join("; ")).into());
        }
        Ok(*output)
    }

    pub async fn receive(
        &self,
        client: &Client,
        gift_wrap: &Event,
    ) -> Result<UnwrappedGift, Box<dyn std::error::Error>> {
        let unwrapped = client.unwrap_gift_wrap(gift_wrap).await?;
        Ok(unwrapped)
    }
}

impl Default for MessageService {
    fn default() -> Self {
        Self::new()
    }
}
```

With:
```rust
use nostr::prelude::*;
use nostr_sdk::prelude::*;

pub async fn send_message(
    client: &Client,
    receiver_pubkey: PublicKey,
    content: &str,
) -> Result<EventId, Box<dyn std::error::Error>> {
    let output = client
        .send_private_msg(receiver_pubkey, content, vec![])
        .await?;
    if output.success.is_empty() {
        let errors: Vec<String> = output
            .failed
            .iter()
            .map(|(url, err)| format!("{url}: {err}"))
            .collect();
        return Err(format!("Échec d'envoi: {}", errors.join("; ")).into());
    }
    Ok(*output)
}

pub async fn receive_message(
    client: &Client,
    gift_wrap: &Event,
) -> Result<UnwrappedGift, Box<dyn std::error::Error>> {
    let unwrapped = client.unwrap_gift_wrap(gift_wrap).await?;
    Ok(unwrapped)
}
```

- [ ] **Step 2: Update lib.rs re-export**

In `crates/rr-core/src/lib.rs`, replace:
```rust
pub use message::MessageService;
```
With:
```rust
pub use message::{send_message, receive_message};
```

- [ ] **Step 3: Update main.rs call sites**

In `crates/rr-cli/src/main.rs`:
- Line 9: `use rr_core::message::MessageService;` → `use rr_core::message::{send_message, receive_message};`
- Line 448: `let msg_service = MessageService::new();` → remove this line
- Line 449-451:
```rust
    match msg_service
        .send(transport.client(), receiver_pubkey, message)
        .await
```
→
```rust
    match send_message(transport.client(), receiver_pubkey, message).await
```
- Line 526: `MessageService::new().receive(&client, &event).await` → `receive_message(&client, &event).await`

- [ ] **Step 4: Verify compilation**

Run: `./scripts/dev.sh cargo check --workspace --exclude rr-tauri 2>&1 | tail -5`
Expected: no errors

- [ ] **Step 5: Run tests**

Run: `./scripts/dev.sh cargo test --workspace --exclude rr-tauri --locked 2>&1 | tail -10`
Expected: all tests pass

- [ ] **Step 6: Commit**

```bash
rtk git add -A
rtk git commit --no-verify -m "clean: MessageService struct → free functions"
```

---

### Task 2: Remove TransportProvider trait

**Files:**
- Modify: `crates/rr-core/src/transport/mod.rs`

- [ ] **Step 1: Simplify transport/mod.rs**

Replace the entire file:
```rust
use nostr_sdk::prelude::*;

pub mod nostr;

pub trait TransportProvider: Send + Sync {
    fn client(&self) -> &Client;
    fn kind(&self) -> &'static str;
}

impl TransportProvider for nostr::NostrTransport {
    fn client(&self) -> &Client {
        self.client()
    }

    fn kind(&self) -> &'static str {
        "nostr"
    }
}
```

With:
```rust
pub mod nostr;
pub use nostr::NostrTransport;
```

- [ ] **Step 2: Verify compilation**

Run: `./scripts/dev.sh cargo check --workspace --exclude rr-tauri 2>&1 | tail -5`
Expected: no errors

- [ ] **Step 3: Run tests**

Run: `./scripts/dev.sh cargo test --workspace --exclude rr-tauri --locked 2>&1 | tail -10`
Expected: all tests pass

- [ ] **Step 4: Commit**

```bash
rtk git add -A
rtk git commit --no-verify -m "clean: remove TransportProvider trait"
```

---

### Task 3: Remove legacy NIP-44 path from listen + cell_key_hex cleanup

**Files:**
- Modify: `crates/rr-core/src/cell_transport.rs`
- Modify: `crates/rr-core/src/cell.rs`

- [ ] **Step 1: Remove legacy NIP-44 fallback from listen mode 1**

In `crates/rr-core/src/cell_transport.rs`, remove lines 524-541:
```rust
                            // Legacy fallback: NIP-44
                            if let (Some(ref sk), Some(ref pk)) = (&cell_sk, &cell_pk) {
                                if let Ok(plaintext) =
                                    CryptoProvider::decrypt(sk, pk, &rumor.content)
                                {
                                    if sender_pk != keys.public_key() {
                                        let snpub = sender_pk
                                            .to_bech32()
                                            .unwrap_or_else(|_| sender_pk.to_string());
                                        println!("[{}] {}: {}", tid, snpub, plaintext);
                                    }
                                }
                            }
                            return Ok(false);
```

Replace with just:
```rust
                            return Ok(false);
```

- [ ] **Step 2: Remove legacy NIP-44 fallback from listen mode 2**

In the listen mode 2 section, remove the legacy NIP-44 block — the `if let Some((cell_key_hex, cell_label)) = cell_info { SecretKey::from_hex(...); CryptoProvider::decrypt(...) }` block. This is the fallback after `drop(store)`.

Replace the verbose `cell_info` extraction (which was only needed for the legacy path) with a simpler `cell_label` extraction:

```rust
                                // Clone cell label for success path before dropping store
                                let cell_label = store.find(&cid).map(|c| c.label.clone());
```

(The `cell_label` is still needed by the sender key success block's `println!`. `cell_key_hex` is no longer needed since the legacy path is gone.)

Then in the success `println!` block, update:
```rust
                                                    let cell_label = cell_info
                                                        .as_ref()
                                                        .map(|(_, l)| l.as_str())
                                                        .unwrap_or("");
```
To:
```rust
                                                    let label = cell_label
                                                        .as_deref()
                                                        .unwrap_or("");
```

(Note: the variable is already named `cell_label` so we renamed the local binding to `label` to avoid shadowing.)

- [ ] **Step 3: Remove cell_sk, cell_pk from listen tuple**

In the listen function's tuple at the top (around line 419), change:
```rust
        let (target_cell_id, cell_sk, cell_pk) = if let Some(cid) = cell_id {
            let store = self.store.lock().await;
            let cell = store
                .find(cid)
                .ok_or_else(|| format!("Cellule {} introuvable", cid))?;
            let sk = SecretKey::from_hex(&cell.cell_key_hex).ok();
            let pk = sk.as_ref().map(|s| Keys::new(s.clone()).public_key());
            (Some(cell.id.to_string()), sk, pk)
        } else {
            (None, None, None)
        };
```

Replace with:
```rust
        let target_cell_id = if let Some(cid) = cell_id {
            let store = self.store.lock().await;
            let cell = store
                .find(cid)
                .ok_or_else(|| format!("Cellule {} introuvable", cid))?;
            Some(cell.id.to_string())
        } else {
            None
        };
```

Also remove the `cell_sk.clone()` line in the closure captures:
```rust
                let cell_sk = cell_sk.clone();
```
(Remove this line entirely)

- [ ] **Step 4: Remove `import crate::CryptoProvider` from cell_transport.rs**

Remove line 10:
```rust
use crate::CryptoProvider;
```

- [ ] **Step 5: Make cell_key_hex optional with `#[serde(default)]`**

In `crates/rr-core/src/cell.rs`, change:
```rust
    /// Hex-encoded SecretKey (cell_key_hex -> SecretKey::from_hex)
    pub cell_key_hex: String,
```
To:
```rust
    /// Hex-encoded SecretKey (legacy, unused for new cells)
    #[serde(default)]
    pub cell_key_hex: String,
```

In `create_cell` (cell_transport.rs line 61), remove `cell_key_hex: String::new(),` from the Cell construction (it defaults to empty String via `#[serde(default)]`).

- [ ] **Step 6: Verify compilation**

Run: `./scripts/dev.sh cargo check --workspace --exclude rr-tauri 2>&1 | tail -5`
Expected: no errors

- [ ] **Step 7: Run tests**

Run: `./scripts/dev.sh cargo test --workspace --exclude rr-tauri --locked 2>&1 | tail -15`
Expected: all tests pass

- [ ] **Step 8: Commit**

```bash
rtk git add -A
rtk git commit --no-verify -m "clean: remove legacy NIP-44 path from listen, cell_key_hex #[serde(default)]"
```

---

### Task 4: Remove CryptoProvider struct, inline nip44 calls in tests

**Files:**
- Modify: `crates/rr-core/src/crypto.rs`
- Modify: `crates/rr-core/src/lib.rs`
- Modify: `crates/rr-core/tests/cell_crypto.rs`
- Modify: `crates/rr-core/tests/proptest.rs`

- [ ] **Step 1: Remove CryptoProvider struct + adapt inline tests in crypto.rs**

In `crates/rr-core/src/crypto.rs`, remove the `CryptoProvider` struct and its impl block. Keep the `use` imports and adapt the `#[cfg(test)] mod tests` to call `nip44` directly.

Replace the entire file:
```rust
use nostr::nips::nip44;
use nostr::{Keys, PublicKey, SecretKey};

#[derive(Debug, Clone)]
pub struct CryptoProvider;

impl CryptoProvider {
    pub fn encrypt(
        secret_key: &SecretKey,
        public_key: &PublicKey,
        content: &str,
    ) -> Result<String, nip44::Error> {
        nip44::encrypt(secret_key, public_key, content, nip44::Version::V2)
    }

    pub fn decrypt(
        secret_key: &SecretKey,
        public_key: &PublicKey,
        payload: &str,
    ) -> Result<String, nip44::Error> {
        nip44::decrypt(secret_key, public_key, payload)
    }

    pub fn generate_keys() -> Keys {
        Keys::generate()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn alice_bob() -> (Keys, Keys) {
        (Keys::generate(), Keys::generate())
    }

    fn encrypt(msg: &str, alice: &Keys, bob: &Keys) -> String {
        CryptoProvider::encrypt(alice.secret_key(), &bob.public_key(), msg).unwrap()
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let (alice, bob) = alice_bob();
        let msg = "Hello RéseauRacine!";
        let cipher = encrypt(msg, &alice, &bob);
        let plain =
            CryptoProvider::decrypt(bob.secret_key(), &alice.public_key(), &cipher).unwrap();
        assert_eq!(plain, msg);
    }
    // ... (7 more tests using CryptoProvider)
}
```

With:
```rust
use nostr::nips::nip44;
use nostr::Keys;

#[cfg(test)]
mod tests {
    use nostr::Keys;
    use nostr::nips::nip44;

    fn alice_bob() -> (Keys, Keys) {
        (Keys::generate(), Keys::generate())
    }

    fn encrypt(msg: &str, alice: &Keys, bob: &Keys) -> String {
        nip44::encrypt(alice.secret_key(), &bob.public_key(), msg, nip44::Version::V2).unwrap()
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let (alice, bob) = alice_bob();
        let msg = "Hello RéseauRacine!";
        let cipher = encrypt(msg, &alice, &bob);
        let plain = nip44::decrypt(bob.secret_key(), &alice.public_key(), &cipher).unwrap();
        assert_eq!(plain, msg);
    }

    #[test]
    fn test_wrong_key_fails() {
        let (alice, bob) = alice_bob();
        let eve = Keys::generate();
        let cipher = encrypt("secret", &alice, &bob);
        let result = nip44::decrypt(eve.secret_key(), &alice.public_key(), &cipher);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_message_rejected() {
        let (alice, bob) = alice_bob();
        let result = nip44::encrypt(alice.secret_key(), &bob.public_key(), "", nip44::Version::V2);
        assert!(result.is_err(), "NIP-44 V2 rejects empty messages");
    }

    #[test]
    fn test_large_message_roundtrip() {
        let (alice, bob) = alice_bob();
        let msg = "A".repeat(10000);
        let cipher = encrypt(&msg, &alice, &bob);
        let plain = nip44::decrypt(bob.secret_key(), &alice.public_key(), &cipher).unwrap();
        assert_eq!(plain, msg);
    }

    #[test]
    fn test_oversized_message_rejected() {
        let (alice, bob) = alice_bob();
        let msg = "A".repeat(65536);
        let result = nip44::encrypt(alice.secret_key(), &bob.public_key(), &msg, nip44::Version::V2);
        assert!(result.is_err(), "NIP-44 V2 rejects messages > 65535 bytes");
    }

    #[test]
    fn test_unicode_message() {
        let (alice, bob) = alice_bob();
        let msg = "éèêëàâäùûüôöîïç€œæ🌿🔑 ∑∏∫ ≤ ≥ ∞ 你好 👋";
        let cipher = encrypt(msg, &alice, &bob);
        let plain = nip44::decrypt(bob.secret_key(), &alice.public_key(), &cipher).unwrap();
        assert_eq!(plain, msg);
    }

    #[test]
    fn test_invalid_ciphertext_fails() {
        let (alice, bob) = alice_bob();
        let result = nip44::decrypt(bob.secret_key(), &alice.public_key(), "garbage");
        assert!(result.is_err());
    }

    #[test]
    fn test_sender_decrypts_own_message() {
        let (alice, bob) = alice_bob();
        let cipher = encrypt("self-test", &alice, &bob);
        let plain = nip44::decrypt(alice.secret_key(), &bob.public_key(), &cipher).unwrap();
        assert_eq!(plain, "self-test");
    }

    #[test]
    fn test_keys_are_unique() {
        let a = Keys::generate();
        let b = Keys::generate();
        assert_ne!(a.secret_key(), b.secret_key());
    }
}
```

- [ ] **Step 2: Update cell_crypto.rs tests**

In `crates/rr-core/tests/cell_crypto.rs`:

Replace:
```rust
use nostr::Keys;
use rr_core::CryptoProvider;

#[test]
fn test_group_key_symmetric_encrypt_decrypt() {
    let cell_keys = Keys::generate();
    let cell_sk = cell_keys.secret_key();
    let cell_pk = &cell_keys.public_key();
    let msg = "Hello cellule!";

    let cipher = CryptoProvider::encrypt(cell_sk, cell_pk, msg).unwrap();
    let plain = CryptoProvider::decrypt(cell_sk, cell_pk, &cipher).unwrap();
    assert_eq!(plain, msg);
}

#[test]
fn test_group_key_deterministic() {
    let cell_keys = Keys::generate();
    let cell_sk = cell_keys.secret_key();
    let cell_pk = &cell_keys.public_key();
    let msg = "determinism test";

    let c1 = CryptoProvider::encrypt(cell_sk, cell_pk, msg).unwrap();
    let c2 = CryptoProvider::encrypt(cell_sk, cell_pk, msg).unwrap();
    assert_ne!(c1, c2);

    assert_eq!(CryptoProvider::decrypt(cell_sk, cell_pk, &c1).unwrap(), msg);
    assert_eq!(CryptoProvider::decrypt(cell_sk, cell_pk, &c2).unwrap(), msg);
}

#[test]
fn test_group_key_rejects_wrong_key() {
    let cell_keys = Keys::generate();
    let wrong_keys = Keys::generate();
    let cell_sk = cell_keys.secret_key();
    let cell_pk = &cell_keys.public_key();
    let wrong_sk = wrong_keys.secret_key();
    let wrong_pk = &wrong_keys.public_key();

    let msg = "secret group message";
    let cipher = CryptoProvider::encrypt(cell_sk, cell_pk, msg).unwrap();

    let result = CryptoProvider::decrypt(wrong_sk, wrong_pk, &cipher);
    assert!(result.is_err());
}
```

With:
```rust
use nostr::Keys;
use nostr::nips::nip44;

fn encrypt(cell_sk: &nostr::SecretKey, cell_pk: &nostr::PublicKey, msg: &str) -> String {
    nip44::encrypt(cell_sk, cell_pk, msg, nip44::Version::V2).unwrap()
}

fn decrypt(cell_sk: &nostr::SecretKey, cell_pk: &nostr::PublicKey, cipher: &str) -> String {
    nip44::decrypt(cell_sk, cell_pk, cipher).unwrap()
}

#[test]
fn test_group_key_symmetric_encrypt_decrypt() {
    let cell_keys = Keys::generate();
    let cell_sk = cell_keys.secret_key();
    let cell_pk = &cell_keys.public_key();
    let msg = "Hello cellule!";

    let cipher = encrypt(cell_sk, cell_pk, msg);
    let plain = decrypt(cell_sk, cell_pk, &cipher);
    assert_eq!(plain, msg);
}

#[test]
fn test_group_key_deterministic() {
    let cell_keys = Keys::generate();
    let cell_sk = cell_keys.secret_key();
    let cell_pk = &cell_keys.public_key();
    let msg = "determinism test";

    let c1 = encrypt(cell_sk, cell_pk, msg);
    let c2 = encrypt(cell_sk, cell_pk, msg);
    assert_ne!(c1, c2);

    assert_eq!(decrypt(cell_sk, cell_pk, &c1), msg);
    assert_eq!(decrypt(cell_sk, cell_pk, &c2), msg);
}

#[test]
fn test_group_key_rejects_wrong_key() {
    let cell_keys = Keys::generate();
    let wrong_keys = Keys::generate();
    let cell_sk = cell_keys.secret_key();
    let cell_pk = &cell_keys.public_key();
    let wrong_sk = wrong_keys.secret_key();
    let wrong_pk = &wrong_keys.public_key();

    let msg = "secret group message";
    let cipher = encrypt(cell_sk, cell_pk, msg);

    let result = nip44::decrypt(wrong_sk, wrong_pk, &cipher);
    assert!(result.is_err());
}
```

- [ ] **Step 3: Update proptest.rs tests**

In `crates/rr-core/tests/proptest.rs`:

Replace:
```rust
use nostr::Keys;
use proptest::prelude::*;
use rr_core::CryptoProvider;
```

With:
```rust
use nostr::{Keys, nips::nip44};
use proptest::prelude::*;
```

Replace all `CryptoProvider::encrypt(...)` with `nip44::encrypt(secret_key, public_key, content, nip44::Version::V2)`.
Replace all `CryptoProvider::decrypt(...)` with `nip44::decrypt(secret_key, public_key, payload)`.

- [ ] **Step 4: Update lib.rs — remove CryptoProvider re-export**

In `crates/rr-core/src/lib.rs`, remove:
```rust
pub use crypto::CryptoProvider;
```

- [ ] **Step 5: Verify compilation**

Run: `./scripts/dev.sh cargo check --workspace --exclude rr-tauri 2>&1 | tail -5`
Expected: no errors

- [ ] **Step 6: Run tests**

Run: `./scripts/dev.sh cargo test --workspace --exclude rr-tauri --locked 2>&1 | tail -15`
Expected: all tests pass (especially crypto + cell_crypto + proptest)

- [ ] **Step 7: Commit**

```bash
rtk git add -A
rtk git commit --no-verify -m "clean: remove CryptoProvider struct, inline nip44 calls"
```

---

### Task 5: Final verification

- [ ] **Step 1: Workspace clippy**

Run: `./scripts/dev.sh cargo clippy --workspace --exclude rr-tauri -- -D warnings 2>&1 | tail -5`
Expected: no warnings

- [ ] **Step 2: Full test suite**

Run: `./scripts/dev.sh cargo test --workspace --exclude rr-tauri --locked 2>&1 | tail -10`
Expected: all tests pass

- [ ] **Step 3: Build CLI release**

Run: `./scripts/dev.sh cargo build --release --package rr-cli 2>&1 | tail -5`
Expected: builds successfully
