# EPIC 1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Alice et Bob échangent un message chiffré E2E via NIP-17 sur relais Nostr public.

**Architecture:** message.rs wrappe `nostr-sdk::Client` (send_private_msg / unwrap_gift_wrap). CLI `send` et `sync` chargent l'identité, créent un transport avec les vraies clés, opèrent. Le bot example de nostr-sdk montre le pattern exact pour la réception (subscribe + handle_notifications).

**Tech Stack:** nostr-sdk 0.44.1 (Client, send_private_msg, unwrap_gift_wrap), nostr 0.44.3 (nip59, UnwrappedGift), tokio, clap 4.6.

---

### Task 1: MessageService final (library)

**Files:**
- Modify: `crates/rr-core/src/message.rs`

Le code actuel est déjà quasi correct. On le finalise : `send()` retourne `EventId`, `receive()` retourne `UnwrappedGift`. Aucun test unitaire nécessaire — le vrai test est l'intégration e2e (Task 3-4).

- [ ] **Step 1: Finaliser message.rs**

Remplacer le contenu par :

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
```

- [ ] **Step 2: Commit**

```bash
git add crates/rr-core/src/message.rs
git commit -m "feat: finalize message service with NIP-17 send/receive"
```

---

### Task 2: Transport minor improvement

**Files:**
- Modify: `crates/rr-core/src/transport/nostr.rs`

Ajouter un getter `relay_url()` pour que le CLI puisse afficher où le message est envoyé.

- [ ] **Step 1: Ajouter relay_url getter**

Dans `NostrTransport`, ajouter un champ `relay_url: String` et un getter :

```rust
use nostr_sdk::prelude::*;

#[derive(Debug, Clone)]
pub struct NostrTransport {
    client: Client,
    relay_url: String,
}

impl NostrTransport {
    pub async fn new(relay_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let keys = Keys::generate();
        let client = Client::new(keys);
        client.add_relay(relay_url).await?;
        client.connect().await;
        Ok(Self { client, relay_url: relay_url.to_string() })
    }

    pub async fn with_keys(
        relay_url: &str,
        keys: Keys,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let client = Client::new(keys);
        client.add_relay(relay_url).await?;
        client.connect().await;
        Ok(Self { client, relay_url: relay_url.to_string() })
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub fn relay_url(&self) -> &str {
        &self.relay_url
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add crates/rr-core/src/transport/nostr.rs
git commit -m "feat(transport): add relay_url getter"
```

---

### Task 3: CLI `rr send`

**Files:**
- Modify: `crates/rr-cli/src/main.rs`

Implémenter la commande `send` : charger l'identité → résoudre le contact → créer transport → envoyer.

- [ ] **Step 1: Implémenter cmd_send**

Remplacer la fonction `cmd_send` :

```rust
async fn cmd_send(contact: &str, message: &str) {
    let data_dir = rr_core::identity::IdentityManager::default_data_dir();

    // Charger l'identité
    let manager = rr_core::identity::IdentityManager::new(&data_dir);
    let identity = match manager.load() {
        Ok(id) => id,
        Err(e) => {
            eprintln!("Erreur: aucune identité trouvée (lancez `rr init`) : {}", e);
            return;
        }
    };

    // Résoudre le contact
    let contacts_path = data_dir.join("contacts.json");
    let contacts: Vec<serde_json::Value> = if contacts_path.exists() {
        match std::fs::read_to_string(&contacts_path) {
            Ok(c) => serde_json::from_str(&c).unwrap_or_default(),
            Err(_) => vec![],
        }
    } else {
        vec![]
    };
    let receiver_npub = match contacts.iter().find(|c| c["name"] == contact) {
        Some(c) => c["npub"].as_str().unwrap(),
        None => {
            eprintln!("Erreur: contact '{}' non trouvé. Ajoutez-le avec `rr add-contact`", contact);
            return;
        }
    };
    let receiver_pubkey = match PublicKey::from_bech32(receiver_npub) {
        Ok(pk) => pk,
        Err(e) => {
            eprintln!("Erreur: npub invalide pour '{}': {}", contact, e);
            return;
        }
    };

    // Connexion au relais
    let relay = std::env::var("RR_RELAY").unwrap_or_else(|_| "wss://relay.damus.io".to_string());
    let transport = match NostrTransport::with_keys(&relay, identity.keys().clone()).await {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Erreur connexion au relais {}: {}", relay, e);
            return;
        }
    };

    // Envoyer
    let msg_service = MessageService::new();
    match msg_service.send(transport.client(), receiver_pubkey, message).await {
        Ok(event_id) => {
            println!("✅ Message envoyé à {} sur {}", contact, relay);
            println!("   Event ID: {}", event_id.to_hex());
        }
        Err(e) => {
            eprintln!("Erreur envoi message: {}", e);
        }
    }
}
```

Ajouter les imports nécessaires en haut de `main.rs` :

```rust
use nostr::nips::nip19::FromBech32;
use nostr::PublicKey;
use rr_core::message::MessageService;
use rr_core::transport::nostr::NostrTransport;
```

- [ ] **Step 2: Vérifier que ça compile**

```bash
./scripts/dev.sh cargo check --package rr-cli
```

Expected: compilation réussie.

- [ ] **Step 3: Commit**

```bash
git add crates/rr-cli/src/main.rs
git commit -m "feat(cli): implement rr send with NIP-17"
```

---

### Task 4: CLI `rr sync`

**Files:**
- Modify: `crates/rr-cli/src/main.rs`

Implémenter la commande `sync` : charger l'identité → se connecter → s'abonner aux events kind 1059 → unwrap → afficher.

- [ ] **Step 1: Implémenter cmd_sync**

Basé sur le pattern exact du bot example nostr-sdk (nostr-sdk/examples/bot.rs:36-68).

Remplacer `cmd_sync` :

```rust
async fn cmd_sync() {
    let data_dir = rr_core::identity::IdentityManager::default_data_dir();

    // Charger l'identité
    let manager = rr_core::identity::IdentityManager::new(&data_dir);
    let identity = match manager.load() {
        Ok(id) => id,
        Err(e) => {
            eprintln!("Erreur: aucune identité trouvée (lancez `rr init`) : {}", e);
            return;
        }
    };

    // Connexion au relais
    let relay = std::env::var("RR_RELAY").unwrap_or_else(|_| "wss://relay.damus.io".to_string());
    let transport = match NostrTransport::with_keys(&relay, identity.keys().clone()).await {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Erreur connexion au relais {}: {}", relay, e);
            return;
        }
    };

    println!("🔄 Connecté à {}, synchronisation...", relay);
    let client = transport.client().clone();

    // Charger les contacts pour résoudre npub → nom
    let contacts_path = data_dir.join("contacts.json");
    let contacts: Vec<serde_json::Value> = if contacts_path.exists() {
        std::fs::read_to_string(&contacts_path)
            .ok()
            .and_then(|c| serde_json::from_str(&c).ok())
            .unwrap_or_default()
    } else {
        vec![]
    };

    // S'abonner aux GiftWrap pour notre pubkey
    let subscription = Filter::new()
        .kind(Kind::GiftWrap)
        .pubkey(identity.public_key())
        .limit(0); // limit 0 = only new events (gift wrap timestamps are tweaked)

    if let Err(e) = client.subscribe(subscription, None).await {
        eprintln!("Erreur abonnement: {}", e);
        return;
    }

    let mut count = 0u32;
    if let Err(e) = client
        .handle_notifications(|notification| {
            let contacts = &contacts;
            let mut count = &mut count;
            async move {
                if let RelayPoolNotification::Event { event, .. } = notification {
                    if event.kind == Kind::GiftWrap {
                        match client.unwrap_gift_wrap(&event).await {
                            Ok(UnwrappedGift { rumor, sender }) => {
                                if rumor.kind == Kind::PrivateDirectMessage {
                                    *count += 1;
                                    let sender_npub = sender.to_bech32().unwrap_or_else(|_| sender.to_string());
                                    let sender_name = contacts
                                        .iter()
                                        .find(|c| c["npub"] == sender_npub)
                                        .and_then(|c| c["name"].as_str())
                                        .unwrap_or(&sender_npub);
                                    println!("📨 {}: {}", sender_name, rumor.content);
                                }
                            }
                            Err(e) => eprintln!("⚠️  Erreur déchiffrement: {}", e),
                        }
                    }
                }
                Ok(false) // continue listening
            }
        })
        .await
    {
        eprintln!("Erreur notification loop: {}", e);
        return;
    }

    if count == 0 {
        println!("📭 Aucun nouveau message.");
    }
}
```

Ajouter l'import si pas déjà présent :

```rust
use nostr::nips::nip19::ToBech32;
```

- [ ] **Step 2: Vérifier que ça compile**

```bash
./scripts/dev.sh cargo check --package rr-cli
```

Expected: compilation réussie.

- [ ] **Step 3: Commit**

```bash
git add crates/rr-cli/src/main.rs
git commit -m "feat(cli): implement rr sync with NIP-17 subscription"
```

---

### Task 5: Test e2e local

**Files:** aucun (test manuel)

- [ ] **Step 1: Lancer le relais local**

```bash
docker compose -f .devcontainer/compose.yaml up -d nostr-relay
```

Vérifier : `curl -s http://localhost:8080` → OK.

- [ ] **Step 2: Créer deux identités**

```bash
./scripts/dev.sh cargo run --package rr-cli -- init
# → noter npub Alice
./scripts/dev.sh env RR_RELAY=ws://localhost:8080 cargo run --package rr-cli -- init
# Oh, ça écrase. Il faut deux data dirs différents.
```

Alternative pour test e2e sans deux machines : utiliser `RR_DATA_DIR` ou `--data-dir`.

Ajouter le support d'une variable d'env `RR_DATA_DIR` dans `cmd_init`, `cmd_send`, `cmd_sync` :

```rust
fn data_dir() -> PathBuf {
    std::env::var("RR_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| rr_core::identity::IdentityManager::default_data_dir())
}
```

Remplacer tous les `default_data_dir()` par `data_dir()` dans les commandes.

Commits séparé :

```bash
# Après modification
git add crates/rr-cli/src/main.rs
git commit -m "feat(cli): support RR_DATA_DIR for multi-identity test"
```

- [ ] **Step 3: Test Alice → Bob via relais local**

```bash
# Terminal 1 — Alice
export RR_DATA_DIR=/tmp/rr-alice
export RR_RELAY=ws://localhost:8080
./scripts/dev.sh cargo run --package rr-cli -- init
./scripts/dev.sh cargo run --package rr-cli -- add-contact <npub_bob> bob
./scripts/dev.sh cargo run --package rr-cli -- send bob "Salut, ça marche"

# Terminal 2 — Bob
export RR_DATA_DIR=/tmp/rr-bob
export RR_RELAY=ws://localhost:8080
./scripts/dev.sh cargo run --package rr-cli -- init
./scripts/dev.sh cargo run --package rr-cli -- add-contact <npub_alice> alice
./scripts/dev.sh cargo run --package rr-cli -- sync
```

Expected: Bob voit `📨 alice: Salut, ça marche`.

---

### Self-review

**Spec coverage:**
- [x] message.rs NIP-17 send/receive → Task 1
- [x] transport relay_url → Task 2
- [x] CLI send (load identity, resolve contact, send) → Task 3
- [x] CLI sync (subscribe, unwrap, display) → Task 4
- [x] Config relais (RR_RELAY) → Tasks 3-4
- [x] Test e2e → Task 5
- [ ] Config relais via fichier config.toml — NON inclus (YAGNI, RR_RELAY suffit)

**Placeholder scan:** aucun.

**Type consistency:** `MessageService::send` retourne `EventId`, `receive` retourne `UnwrappedGuild`. Cohérent. `NostrTransport` a `relay_url()` retourne `&str`. `cmd_send` et `cmd_sync` utilisent `NostrTransport::with_keys(relay, identity.keys().clone())`.
