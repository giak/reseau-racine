# EPIC 2 — Groupes & Cellules Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Allow small groups (cellules, 3 members, extensible to 10) to communicate with E2EE on Nostr using a shared X25519 group key + gift-wrap broadcast.

**Architecture:** NIP-44 encryption with a shared CellKey, gift-wrap (kind 1059) broadcast per member, `h` tag for group routing. Key distribution in-band via gift-wrap DM.

**Tech Stack:** Rust, nostr-sdk 0.44 (nostr 0.44.3), clap 4, serde_json, tokio, uuid

---

### Task 0: Verify NIP-44 symmetric key assumption

**Files:**
- Create: `crates/rr-core/tests/cell_crypto.rs`

- [ ] **Step 1: Write test**

```rust
use nostr::Keys;
use rr_core::CryptoProvider;

#[test]
fn test_group_key_symmetric_encrypt_decrypt() {
    let cell_keys = Keys::generate();
    let cell_sk = cell_keys.secret_key();
    let cell_pk = &cell_keys.public_key();
    let msg = "Hello cellule!";

    // NIP-44 encrypt/decrypt with the same keypair (self-DH)
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

    // Two encryptions of same message should produce different ciphertexts (nonce)
    let c1 = CryptoProvider::encrypt(cell_sk, cell_pk, msg).unwrap();
    let c2 = CryptoProvider::encrypt(cell_sk, cell_pk, msg).unwrap();
    assert_ne!(c1, c2);

    // Both should decrypt correctly
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

    // Wrong key should fail
    let result = CryptoProvider::decrypt(wrong_sk, wrong_pk, &cipher);
    assert!(result.is_err());
}
```

- [ ] **Step 2: Run**

Run: `./scripts/dev.sh cargo test test_group_key_symmetric --package rr-core`

Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add crates/rr-core/tests/cell_crypto.rs
git commit -m "test: NIP-44 symmetric key self-DH encrypt/decrypt for groups"
```

---

### Task 1: Cell types + CellStore

**Files:**
- Create: `crates/rr-core/src/cell.rs`
- Modify: `crates/rr-core/src/lib.rs`
- Modify: `crates/rr-core/Cargo.toml` (add uuid)
- Create: `crates/rr-core/tests/cell_store.rs`

**Note:** `SecretKey` has no `Serialize` impl (security). Store `cell_key_hex: String`, convert via `SecretKey::from_hex` on load.

- [ ] **Step 1: Add uuid dep**

```toml
# crates/rr-core/Cargo.toml — add to [dependencies]
uuid = { version = "1", features = ["serde", "v4"] }
```

- [ ] **Step 2: Write cell.rs**

```rust
use nostr::PublicKey;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellMember {
    pub pubkey: PublicKey,
    pub label: Option<String>,
    pub added_at_secs: u64,
}

impl CellMember {
    pub fn new(pubkey: PublicKey, label: Option<String>) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self { pubkey, label, added_at_secs: now }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cell {
    pub id: Uuid,
    pub label: String,
    /// Hex-encoded SecretKey (cell_key_hex -> SecretKey::from_hex)
    pub cell_key_hex: String,
    pub members: Vec<CellMember>,
    pub created_at_secs: u64,
}

impl Cell {
    pub fn new(label: &str, cell_key_hex: String, members: Vec<CellMember>) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            id: Uuid::new_v4(),
            label: label.to_string(),
            cell_key_hex,
            members,
            created_at_secs: now,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CellStore {
    cells: Vec<Cell>,
}

impl CellStore {
    pub fn path() -> PathBuf {
        crate::config::Config::config_dir().join("cells.json")
    }

    pub fn load() -> Self {
        let path = Self::path();
        if !path.exists() {
            return Self::default();
        }
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|c| serde_json::from_str(&c).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let dir = Self::path().parent().unwrap();
        std::fs::create_dir_all(dir)?;
        std::fs::write(Self::path(), serde_json::to_string_pretty(self)?)?;
        Ok(())
    }

    pub fn find(&self, id: &Uuid) -> Option<&Cell> {
        self.cells.iter().find(|c| &c.id == id)
    }

    pub fn all(&self) -> &[Cell] {
        &self.cells
    }

    pub fn add(&mut self, cell: Cell) {
        self.cells.push(cell);
    }

    pub fn remove(&mut self, id: &Uuid) {
        self.cells.retain(|c| &c.id != id);
    }

    pub fn update_members(&mut self, id: &Uuid, members: Vec<CellMember>) -> bool {
        if let Some(cell) = self.cells.iter_mut().find(|c| &c.id == id) {
            cell.members = members;
            true
        } else {
            false
        }
    }
}
```

- [ ] **Step 3: Register module in lib.rs**

```rust
// crates/rr-core/src/lib.rs — add after existing modules
pub mod cell;
pub use cell::{Cell, CellMember, CellStore};
```

- [ ] **Step 4: Write store tests**

```rust
// crates/rr-core/tests/cell_store.rs
use nostr::PublicKey;
use rr_core::{Cell, CellMember, CellStore};
use std::str::FromStr;
use uuid::Uuid;

fn dummy_pk() -> PublicKey {
    PublicKey::from_str(
        "79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798"
    ).unwrap()
}

#[test]
fn test_cell_roundtrip() {
    let cell = Cell::new(
        "test-cell",
        "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
        vec![CellMember::new(dummy_pk(), Some("Alice".to_string()))],
    );
    let mut store = CellStore::default();
    store.add(cell);
    let json = serde_json::to_string_pretty(&store).unwrap();
    let parsed: CellStore = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.all().len(), 1);
    assert_eq!(parsed.all().first().unwrap().label, "test-cell");
}

#[test]
fn test_cell_store_find() {
    let cell = Cell::new("find-me", "deadbeef", vec![CellMember::new(dummy_pk(), None)]);
    let id = cell.id;
    let mut store = CellStore::default();
    store.add(cell);
    assert!(store.find(&id).is_some());
    assert!(store.find(&Uuid::new_v4()).is_none());
}

#[test]
fn test_cell_store_add_remove() {
    let cell = Cell::new("tmp", "key", vec![CellMember::new(dummy_pk(), None)]);
    let id = cell.id;
    let mut store = CellStore::default();
    store.add(cell);
    assert_eq!(store.all().len(), 1);
    store.remove(&id);
    assert_eq!(store.all().len(), 0);
}

#[test]
fn test_cell_store_update_members() {
    let cell = Cell::new("growing", "key", vec![CellMember::new(dummy_pk(), None)]);
    let id = cell.id;
    let mut store = CellStore::default();
    store.add(cell);
    let new_member = CellMember::new(dummy_pk(), Some("Bob".to_string()));
    assert!(store.update_members(&id, vec![new_member]));
    assert_eq!(store.find(&id).unwrap().members.len(), 1);
    assert_eq!(store.find(&id).unwrap().members[0].label.as_deref(), Some("Bob"));
}
```

- [ ] **Step 5: Run tests**

Run: `./scripts/dev.sh cargo test cell_store --package rr-core`

Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add crates/rr-core/src/cell.rs crates/rr-core/src/lib.rs crates/rr-core/tests/cell_store.rs crates/rr-core/Cargo.toml
git commit -m "feat(core): Cell types + CellStore with serde persistence"
```

---

### Task 2: CellTransport — create + invite

**Files:**
- Create: `crates/rr-core/src/cell_transport.rs`
- Modify: `crates/rr-core/src/lib.rs`

**nostr 0.44.3 APIs in use:**
- `EventBuilder::gift_wrap(signer, &receiver_pk, rumor, []).await?` → `Event`
- `EventBuilder::new(Kind::TextNote, content, tags).to_unsigned_event(pubkey)` → `UnsignedEvent`
- `Tag::custom(TagKind::Custom("h"), vec![cell_id_string])` for group routing
- `client.send_event(event).await?` → `Output<EventId>`
- `Keys::new(sk).public_key()` — derive public key from secret key

- [ ] **Step 0: Add tokio dep to rr-core**

```toml
# crates/rr-core/Cargo.toml — add to [dependencies]
tokio.workspace = true
```

- [ ] **Step 1: Write cell_transport.rs**

```rust
use nostr::prelude::*;
use nostr_sdk::prelude::*;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::cell::{Cell, CellMember, CellStore};
use crate::CryptoProvider;

pub struct CellTransport {
    client: Client,
    keys: Keys,
    store: Arc<Mutex<CellStore>>,
}

impl CellTransport {
    pub fn new(client: Client, keys: Keys) -> Self {
        Self {
            client,
            keys,
            store: Arc::new(Mutex::new(CellStore::load())),
        }
    }

    /// Create a new cell and distribute the CellKey to all members via gift-wrap
    pub async fn create_cell(
        &self,
        label: &str,
        member_pubkeys: &[PublicKey],
    ) -> Result<Cell, Box<dyn std::error::Error>> {
        let cell_keys = Keys::generate();
        let cell_sk_hex = cell_keys.secret_key().to_secret_hex();
        let sender_pk = self.keys.public_key();

        let mut members: Vec<CellMember> = member_pubkeys
            .iter()
            .map(|pk| CellMember::new(*pk, None))
            .collect();
        members.push(CellMember::new(sender_pk, Some("me".to_string())));

        let cell = Cell::new(label, cell_sk_hex, members);
        let cell_id_hex = cell.id.to_string();

        // Distribute CellKey to each member via gift-wrap
        let payload = serde_json::json!({
            "key": cell_sk_hex,
            "label": label,
            "id": cell.id.to_string(),
        });
        let payload_str = payload.to_string();
        for member_pk in member_pubkeys {
            self.send_cell_key(member_pk, &payload_str, &cell_id_hex).await?;
        }

        // Save locally
        let mut store = self.store.lock().await;
        store.add(cell.clone());
        store.save()?;

        Ok(cell)
    }

    /// Add a new member to an existing cell (re-distribute CellKey)
    pub async fn invite_member(
        &self,
        cell_id: &Uuid,
        new_member_pk: &PublicKey,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let store = self.store.lock().await;
        let cell = store.find(cell_id)
            .ok_or_else(|| format!("Cellule {} introuvable", cell_id))?;
        let cell_sk_hex = cell.cell_key_hex.clone();
        let cell_id_hex = cell.id.to_string();
        drop(store);

        self.send_cell_key(new_member_pk, &cell_sk_hex, &cell_id_hex).await?;

        // Add to local store
        let mut store = self.store.lock().await;
        let mut cell = store.find(cell_id).cloned().unwrap();
        cell.members.push(CellMember::new(*new_member_pk, None));
        store.update_members(cell_id, cell.members.clone());
        store.save()?;

        Ok(())
    }

    /// Send CellKey hex to a member via gift-wrap
    async fn send_cell_key(
        &self,
        receiver_pk: &PublicKey,
        cell_key_hex: &str,
        cell_id_hex: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let rumor = EventBuilder::new(
            Kind::TextNote,
            cell_key_hex.to_string(),
            vec![Tag::custom(
                TagKind::Custom("h".to_string()),
                vec![cell_id_hex.to_string()],
            )],
        )
        .to_unsigned_event(self.keys.public_key());

        let wrap = EventBuilder::gift_wrap(&self.keys, receiver_pk, rumor, []).await?;
        self.client.send_event(wrap).await?;
        Ok(())
    }
}
```

- [ ] **Step 2: Register module in lib.rs**

```rust
// crates/rr-core/src/lib.rs — add after cell module
pub mod cell_transport;
pub use cell_transport::CellTransport;
```

- [ ] **Step 3: Write integration test (ignored, needs relay)**

```rust
// crates/rr-core/tests/cell_transport.rs
use nostr::Keys;
use nostr_sdk::Client;
use rr_core::CellTransport;

#[tokio::test]
#[ignore]
async fn test_cell_transport_create() {
    let relay = std::env::var("RR_RELAY").unwrap_or_else(|_| "ws://172.20.0.2:8080".to_string());
    let alice = Keys::generate();
    let bob = Keys::generate();
    let charlie = Keys::generate();

    let client = Client::new(alice.clone());
    client.add_relay(&relay).await.unwrap();
    client.connect().await;

    let transport = CellTransport::new(client, alice.clone());

    let cell = transport.create_cell(
        "test-cell",
        &[bob.public_key(), charlie.public_key()],
    ).await.expect("create_cell failed");

    assert_eq!(cell.label, "test-cell");
    assert_eq!(cell.members.len(), 3);

    let dave = Keys::generate();
    transport.invite_member(&cell.id, &dave.public_key()).await.unwrap();

    let store = rr_core::CellStore::load();
    let loaded = store.find(&cell.id).unwrap();
    assert_eq!(loaded.members.len(), 4);
}
```

- [ ] **Step 4: Check compilation**

Run: `./scripts/dev.sh cargo check --package rr-core`

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/rr-core/src/cell_transport.rs crates/rr-core/src/lib.rs
git commit -m "feat(core): CellTransport create + invite with gift-wrap key distribution"
```

---

### Task 3: CellTransport — send + listen + discover

**Files:**
- Modify: `crates/rr-core/src/cell_transport.rs`

**Key distribution format (JSON in rumor.content):**
```json
{"key": "<cell_sk_hex>", "label": "<cell_label>", "id": "<cell_uuid>"}
```

The `listen` method handles two modes:
- **Specific cell** (`cell_id = Some(id)`) — only shows messages for that cell
- **Discovery** (`cell_id = None`) — scans all gift wraps, auto-creates cells from key distributions, shows messages from all known cells

- [ ] **Step 1: Add send_message + listen methods**

```rust
impl CellTransport {
    /// Send a message to all cell members
    pub async fn send_message(
        &self,
        cell_id: &Uuid,
        content: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let store = self.store.lock().await;
        let cell = store.find(cell_id)
            .ok_or_else(|| format!("Cellule {} introuvable", cell_id))?;

        let cell_sk = SecretKey::from_hex(&cell.cell_key_hex)?;
        let cell_pk = Keys::new(cell_sk.clone()).public_key();
        let cell_id_hex = cell.id.to_string();
        let members: Vec<PublicKey> = cell.members.iter().map(|m| m.pubkey).collect();
        drop(store);

        let encrypted = CryptoProvider::encrypt(&cell_sk, &cell_pk, content)?;

        let rumor = EventBuilder::new(
            Kind::TextNote,
            encrypted,
            vec![Tag::custom(
                TagKind::Custom("h".to_string()),
                vec![cell_id_hex],
            )],
        )
        .to_unsigned_event(self.keys.public_key());

        for member_pk in &members {
            let wrap = EventBuilder::gift_wrap(&self.keys, member_pk, rumor.clone(), []).await?;
            self.client.send_event(wrap).await?;
        }

        Ok(())
    }

    /// Listen for cell messages
    ///
    /// If `cell_id` is `Some`, only processes messages for that specific cell.
    /// If `None`, listens to all gift wraps: auto-creates cells from key
    /// distribution messages and displays messages from known cells.
    pub async fn listen(
        &self,
        cell_id: Option<&Uuid>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let my_pk = self.keys.public_key();
        let client = self.client.clone();

        // If specific cell, pre-load its key
        let (target_cell_id, cell_sk, cell_pk) = if let Some(cid) = cell_id {
            let store = self.store.lock().await;
            let cell = store
                .find(cid)
                .ok_or_else(|| format!("Cellule {} introuvable", cid))?;
            let sk = SecretKey::from_hex(&cell.cell_key_hex)?;
            let pk = Keys::new(sk.clone()).public_key();
            (Some(cell.id.to_string()), Some(sk), Some(pk))
        } else {
            (None, None, None)
        };

        let filter = Filter::new()
            .kind(Kind::GiftWrap)
            .pubkey(my_pk);

        client.subscribe(filter, None).await?;

        if let Some(cid) = &target_cell_id {
            println!("En écoute sur la cellule {} — Ctrl+C pour arrêter", cid);
        } else {
            println!("En écoute (mode découverte) — Ctrl+C pour arrêter");
        }

        client
            .handle_notifications(|notification| {
                let cell_sk = cell_sk.clone();
                let cell_pk = cell_pk;
                let target_cell_id = target_cell_id.clone();
                let client = client.clone();
                let keys = self.keys.clone();
                let store_arc = self.store.clone();

                async move {
                    if let RelayPoolNotification::Event { event, .. } = notification {
                        if event.kind != Kind::GiftWrap {
                            return Ok(false);
                        }
                        let unwrapped = match client.unwrap_gift_wrap(&event).await {
                            Ok(u) => u,
                            Err(_) => return Ok(false),
                        };
                        let rumor = unwrapped.rumor;
                        let sender_pk = unwrapped.sender;

                        // Extract h-tag
                        let h_tag_val: Option<String> = rumor
                            .tags
                            .iter()
                            .find(|t| t.kind() == TagKind::Custom("h".to_string()))
                            .and_then(|t| t.content())
                            .map(|s| s.to_string());

                        let h_tag = match &h_tag_val {
                            Some(v) => v.clone(),
                            None => return Ok(false),
                        };

                        // Mode 1: specific cell
                        if let Some(tid) = &target_cell_id {
                            if &h_tag != tid {
                                return Ok(false);
                            }
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
                        }

                        // Mode 2: discovery — try key distribution first
                        let cell_id_parsed = uuid::Uuid::parse_str(&h_tag);
                        let mut store = store_arc.lock().await;

                        if cell_id_parsed.is_ok() {
                            let cid = cell_id_parsed.unwrap();
                            if store.find(&cid).is_none() {
                                // Unknown cell — check if this is a key distribution
                                if let Ok(payload) =
                                    serde_json::from_str::<serde_json::Value>(&rumor.content)
                                {
                                    if let (Some(key), Some(label)) = (
                                        payload.get("key").and_then(|v| v.as_str()),
                                        payload.get("label").and_then(|v| v.as_str()),
                                    ) {
                                        let new_cell = Cell::new(
                                            label,
                                            key.to_string(),
                                            vec![
                                                CellMember::new(sender_pk, None),
                                                CellMember::new(keys.public_key(), Some("me".to_string())),
                                            ],
                                        );
                                        store.add(new_cell.clone());
                                        if let Err(e) = store.save() {
                                            eprintln!("Erreur sauvegarde cellule: {}", e);
                                        }
                                        println!(
                                            "Nouvelle cellule: {} ({})",
                                            label, new_cell.id
                                        );
                                    }
                                }
                            } else {
                                // Known cell — decrypt and display
                                if let Some(cell) = store.find(&cid).cloned() {
                                    drop(store);
                                    if let Ok(sk) = SecretKey::from_hex(&cell.cell_key_hex) {
                                        let pk = Keys::new(sk.clone()).public_key();
                                        if let Ok(plaintext) =
                                            CryptoProvider::decrypt(&sk, &pk, &rumor.content)
                                        {
                                            if sender_pk != keys.public_key() {
                                                let snpub = sender_pk
                                                    .to_bech32()
                                                    .unwrap_or_else(|_| sender_pk.to_string());
                                                println!("[{}] {}: {}", cell.label, snpub, plaintext);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Ok(false)
                }
            })
            .await?;

        Ok(())
    }
}
```

- [ ] **Step 2: Check compilation**

Run: `./scripts/dev.sh cargo check --package rr-core`

Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add crates/rr-core/src/cell_transport.rs
git commit -m "feat(core): CellTransport send_message + listen with CellKey decrypt"
```

---

### Task 4: CLI group commands

**Files:**
- Modify: `crates/rr-cli/src/main.rs`
- No new deps (uuid, nostr types already available through rr-core)

- [ ] **Step 1: Add Group subcommand to Commands enum**

```rust
// crates/rr-cli/src/main.rs — modify Commands enum
#[derive(Subcommand)]
enum Commands {
    // ... existing commands ...

    /// Commandes de groupe (cellules)
    #[command(subcommand)]
    Group(GroupCommands),
}

#[derive(Subcommand)]
enum GroupCommands {
    /// Créer une cellule
    Create {
        #[arg(long)]
        label: String,
        /// Liste de npubs séparés par des virgules
        #[arg(long, value_delimiter = ',')]
        members: Vec<String>,
    },
    /// Lister les cellules
    List,
    /// Détails d'une cellule
    Info {
        /// Cell ID (UUID)
        cell_id: String,
    },
    /// Inviter un membre dans une cellule
    Invite {
        cell_id: String,
        #[arg(long)]
        member: String,
    },
    /// Envoyer un message dans une cellule
    Send {
        cell_id: String,
        #[arg(long)]
        message: String,
    },
    /// Écouter les messages d'une cellule (ou mode découverte sans argument)
    Listen {
        cell_id: Option<String>,
    },
}
```

- [ ] **Step 2: Add match arm in main()**

```rust
// In main(), add to the match:
Commands::Group(group_cmd) => match group_cmd {
    GroupCommands::Create { label, members } => cmd_group_create(label, members).await,
    GroupCommands::List => cmd_group_list().await,
    GroupCommands::Info { cell_id } => cmd_group_info(cell_id).await,
    GroupCommands::Invite { cell_id, member } => cmd_group_invite(cell_id, member).await,
    GroupCommands::Send { cell_id, message } => cmd_group_send(cell_id, message).await,
    GroupCommands::Listen { cell_id } => cmd_group_listen(cell_id.as_deref()).await,
},
```

- [ ] **Step 3: Implement cmd_group_create**

```rust
async fn cmd_group_create(label: &str, members_npub: &[String]) {
    let manager = IdentityManager::new(data_dir()).with_key_source(key_source());
    let identity = match manager.load() {
        Ok(id) => id,
        Err(e) => {
            eprintln!("Erreur: aucune identité trouvée (lancez `rr init`) : {}", e);
            return;
        }
    };

    let mut member_pubkeys = Vec::new();
    for npub in members_npub {
        match PublicKey::from_bech32(npub) {
            Ok(pk) => member_pubkeys.push(pk),
            Err(e) => {
                eprintln!("Erreur: npub invalide '{}': {}", npub, e);
                return;
            }
        }
    }

    if member_pubkeys.is_empty() {
        eprintln!("Erreur: au moins un membre requis");
        return;
    }

    let relay = std::env::var("RR_RELAY").unwrap_or_else(|_| "wss://relay.damus.io".to_string());
    let transport = match NostrTransport::with_keys(&relay, identity.keys().clone()).await {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Erreur connexion relais: {}", e);
            return;
        }
    };

    let cell_transport = CellTransport::new(transport.client().clone(), identity.keys().clone());

    match cell_transport.create_cell(label, &member_pubkeys).await {
        Ok(cell) => {
            println!("✅ Cellule créée : {}", cell.id);
            println!("   Label: {}", cell.label);
            println!("   Membres: {}", cell.members.len());
        }
        Err(e) => eprintln!("Erreur création cellule: {}", e),
    }
}
```

- [ ] **Step 4: Implement cmd_group_list**

```rust
async fn cmd_group_list() {
    let store = CellStore::load();
    let cells = store.all();
    if cells.is_empty() {
        println!("Aucune cellule. Créez-en une avec `rr group create`");
        return;
    }
    for cell in cells {
        println!("  {} — {} ({} membres)", cell.id, cell.label, cell.members.len());
    }
}
```

- [ ] **Step 5: Implement cmd_group_info**

```rust
async fn cmd_group_info(cell_id_str: &str) {
    let cell_id = match Uuid::parse_str(cell_id_str) {
        Ok(id) => id,
        Err(e) => {
            eprintln!("Erreur: UUID invalide '{}': {}", cell_id_str, e);
            return;
        }
    };
    let store = CellStore::load();
    match store.find(&cell_id) {
        Some(cell) => {
            println!("  ID: {}", cell.id);
            println!("  Label: {}", cell.label);
            println!("  Membres:");
            for member in &cell.members {
                let label = member.label.as_deref().unwrap_or("?");
                let npub = member.pubkey.to_bech32().unwrap_or_else(|_| member.pubkey.to_string());
                println!("    • {} ({})", label, npub);
            }
            println!("  Créée le: {}", cell.created_at_secs);
        }
        None => eprintln!("Cellule '{}' introuvable", cell_id_str),
    }
}
```

- [ ] **Step 6: Implement cmd_group_invite**

```rust
async fn cmd_group_invite(cell_id_str: &str, member_npub: &str) {
    let cell_id = match Uuid::parse_str(cell_id_str) {
        Ok(id) => id,
        Err(e) => {
            eprintln!("Erreur: UUID invalide '{}': {}", cell_id_str, e);
            return;
        }
    };
    let member_pk = match PublicKey::from_bech32(member_npub) {
        Ok(pk) => pk,
        Err(e) => {
            eprintln!("Erreur: npub invalide '{}': {}", member_npub, e);
            return;
        }
    };

    let manager = IdentityManager::new(data_dir()).with_key_source(key_source());
    let identity = match manager.load() {
        Ok(id) => id,
        Err(e) => {
            eprintln!("Erreur: identité non trouvée: {}", e);
            return;
        }
    };

    let relay = std::env::var("RR_RELAY").unwrap_or_else(|_| "wss://relay.damus.io".to_string());
    let transport = match NostrTransport::with_keys(&relay, identity.keys().clone()).await {
        Ok(t) => t,
        Err(e) => { eprintln!("Erreur connexion relais: {}", e); return; }
    };

    let cell_transport = CellTransport::new(transport.client().clone(), identity.keys().clone());

    match cell_transport.invite_member(&cell_id, &member_pk).await {
        Ok(()) => println!("✅ Membre invité dans la cellule {}", cell_id),
        Err(e) => eprintln!("Erreur invitation: {}", e),
    }
}
```

- [ ] **Step 7: Implement cmd_group_send**

```rust
async fn cmd_group_send(cell_id_str: &str, message: &str) {
    let cell_id = match Uuid::parse_str(cell_id_str) {
        Ok(id) => id,
        Err(e) => {
            eprintln!("Erreur: UUID invalide '{}': {}", cell_id_str, e);
            return;
        }
    };

    let manager = IdentityManager::new(data_dir()).with_key_source(key_source());
    let identity = match manager.load() {
        Ok(id) => id,
        Err(e) => { eprintln!("Erreur: identité non trouvée: {}", e); return; }
    };

    let relay = std::env::var("RR_RELAY").unwrap_or_else(|_| "wss://relay.damus.io".to_string());
    let transport = match NostrTransport::with_keys(&relay, identity.keys().clone()).await {
        Ok(t) => t,
        Err(e) => { eprintln!("Erreur connexion relais: {}", e); return; }
    };

    let cell_transport = CellTransport::new(transport.client().clone(), identity.keys().clone());

    match cell_transport.send_message(&cell_id, message).await {
        Ok(()) => println!("✅ Message envoyé dans la cellule {}", cell_id),
        Err(e) => eprintln!("Erreur envoi: {}", e),
    }
}
```

- [ ] **Step 8: Implement cmd_group_listen**

```rust
async fn cmd_group_listen(cell_id_str: Option<&str>) {
    let cell_id = match cell_id_str {
        Some(s) => match Uuid::parse_str(s) {
            Ok(id) => Some(id),
            Err(e) => {
                eprintln!("Erreur: UUID invalide '{}': {}", s, e);
                return;
            }
        },
        None => None,
    };

    let manager = IdentityManager::new(data_dir()).with_key_source(key_source());
    let identity = match manager.load() {
        Ok(id) => id,
        Err(e) => { eprintln!("Erreur: identité non trouvée: {}", e); return; }
    };

    let relay = std::env::var("RR_RELAY").unwrap_or_else(|_| "wss://relay.damus.io".to_string());
    let transport = match NostrTransport::with_keys(&relay, identity.keys().clone()).await {
        Ok(t) => t,
        Err(e) => { eprintln!("Erreur connexion relais: {}", e); return; }
    };

    let cell_transport = CellTransport::new(transport.client().clone(), identity.keys().clone());

    if let Err(e) = cell_transport.listen(cell_id.as_ref()).await {
        eprintln!("Erreur écoute: {}", e);
    }
}
```

- [ ] **Step 9: Add `uuid` dep to rr-cli**

```toml
# crates/rr-cli/Cargo.toml — add to [dependencies]
uuid = { version = "1", features = ["v4"] }
```

- [ ] **Step 10: Add imports in main.rs**

```rust
// At top of main.rs — add with existing imports:
use uuid::Uuid;
use rr_core::{CellStore, CellTransport};
```

- [ ] **Step 11: Check compilation**

Run: `./scripts/dev.sh cargo check --package rr-cli`

Expected: PASS

- [ ] **Step 12: Commit**

```bash
git add crates/rr-cli/src/main.rs crates/rr-cli/Cargo.toml
git commit -m "feat(cli): rr group create/list/info/invite/send/listen commands"
```

---

### Task 5: Full integration test (manual)

- [ ] **Step 1: Run the create/list/info cycle**

```bash
# First create the identity
./scripts/dev.sh cargo run --package rr-cli -- init

# Create a cell with two members
# (use real npubs or the user's own npub for self-test)
./scripts/dev.sh cargo run --package rr-cli -- group create --label "test" --members "npub1...,npub2..."

# List cells
./scripts/dev.sh cargo run --package rr-cli -- group list

# Info
./scripts/dev.sh cargo run --package rr-cli -- group info <cell-id>
```

- [ ] **Step 2: Run the send/listen cycle**

```bash
# In terminal 1: listen
./scripts/dev.sh cargo run --package rr-cli -- group listen <cell-id>

# In terminal 2: send
./scripts/dev.sh cargo run --package rr-cli -- group send <cell-id> --message "Hello cellule!"
```

- [ ] **Step 3: Commit any fixes**

```bash
git add -A
git commit -m "fix: integration fixes for group commands"
```
