# EPIC CLEAN-1 — Dead code removal

## Priorité

P2 — Nuit à la lisibilité et à la maintenance, pas de faille de sécurité.

## Problème

Quatre artéfacts de code mort ou redondant identifiés par l'audit :

### 1. `CryptoProvider` — wrapper vide

**Fichier :** `crates/rr-core/src/crypto.rs`

```rust
pub struct CryptoProvider;
impl CryptoProvider {
    pub fn encrypt(...) -> Result<String, nip44::Error> { nip44::encrypt(...) }
    pub fn decrypt(...) -> Result<String, nip44::Error> { nip44::decrypt(...) }
    pub fn generate_keys() -> SecretKey { Keys::generate().secret_key() }
}
```

Délègue directement à `nip44` et `Keys::generate()` sans ajouter de valeur (pas de logging, pas de validation, pas d'abstraction). `generate_keys()` est utilisé dans 1 test. `encrypt`/`decrypt` sont utilisés dans `cell_transport.rs` (legacy path) et dans `tests/cell_crypto.rs`.

**Solution :** Remplacer les appels par les fonctions `nip44` directes. Supprimer le struct, garder les tests en les adaptant pour appeler `nip44` directement.

Ce qui change :
- `cell_transport.rs` : `CryptoProvider::encrypt(sk, pk, msg)` → `nip44::encrypt(sk.secret_bytes(), &pk.xonly_key().serialize(), msg).map_err(|e| format!("{e}"))`
- `tests/cell_crypto.rs` : adapter les 3 tests pour appeler `nip44` directement

**Risque :** `nip44::encrypt` et `nip44::decrypt` sont des fonctions de `nostr::nips::nip44`. Vérifier leur signature exacte avant de modifier.

### 2. `MessageService` — struct inutile

**Fichier :** `crates/rr-core/src/message.rs`

```rust
pub struct MessageService;
impl MessageService {
    pub fn new() -> Self { Self }
    pub async fn send(&self, client: &Client, to: PublicKey, msg: &str) -> Result<...> { ... }
    pub async fn receive(&self, client: &Client, event: &Event) -> Result<...> { ... }
}
```

`send()` n'est jamais appelé depuis le code de production (`main.rs:473` utilise `client.send_private_msg()` directement). `receive()` est appelé une fois dans `main.rs:526` (`MessageService::new().receive(...)`).

**Solution :** Remplacer par des fonctions libres `pub async fn send_message(...)` et `pub async fn receive_message(...)`. Simplifie l'API.

```rust
// message.rs devient :
pub async fn send_message(client: &Client, to: PublicKey, msg: &str) -> Result<..., ...> { ... }
pub async fn receive_message(client: &Client, event: &Event) -> Result<..., ...> { ... }
```

Appels à mettre à jour :
- `main.rs:526` : `MessageService::new().receive(...)` → `rr_core::message::receive_message(...)`
- Tests éventuels

### 3. `TransportProvider` trait — abstraction précoce

**Fichier :** `crates/rr-core/src/transport/mod.rs`

```rust
pub trait TransportProvider {
    fn client(&self) -> &Client;
    fn kind(&self) -> &'static str;
}
```

Trait avec 1 implémentation (`NostrTransport`). Aucun code n'est générique sur `TransportProvider`. Les appelants prennent `&Client` directement. Le trait ne peut pas servir pour un second transport (Reticulum) sans refactor majeur car tout le code dépend déjà de l'API concrète `nostr_sdk::Client`.

**Solution :** Supprimer le trait. Garder `NostrTransport` comme struct concret avec ses méthodes. Supprimer `transport/mod.rs` ou le réduire à un ré-export.

```rust
// transport/mod.rs devient juste :
pub mod nostr;
pub use nostr::NostrTransport;
```

### 4. `cell_key_hex` toujours vide

**Fichier :** `crates/rr-core/src/cell_transport.rs` (create_cell + legacy path)

`create_cell()` met `cell_key_hex: String::new()` (vide intentionnellement). Le legacy path dans `send_message()` appelle `SecretKey::from_hex(&cell.cell_key_hex)` qui échoue sur chaîne vide → legacy path est mort.

**Deux options :**
- **(a) Supprimer le legacy path** — plus propre. Le code Sender Key est seul chemin de déchiffrement. Supprime ~40 lignes de complexité.
- **(b) Documenter que le legacy existe pour backward compat** mais ne peut pas être utilisé pour les nouvelles cellules. Conserver le code mais avec un commentaire explicite.

**Recommandé : (a)** Supprimer le legacy path de `send_message()` et `listen()`. Élimine aussi `cell_key_hex` des types `Cell` (ou le rend optionnel pour lecture de vieilles cellules). Simplifie considérablement le code.

**Choses à creuser :**
- Vérifier qu'aucune cellule existante sur le filesystem de l'utilisateur n'utilise `cell_key_hex`. Si le projet n'a jamais été déployé avec des vraies cellules → safe à supprimer.
- Le champ `cell_key_hex` dans `Cell` peut être conservé avec `#[serde(default)]` pour désérialiser les vieilles cellules sans erreur, mais ne plus l'utiliser en écriture.

## Fichiers impactés

| Fichier | Changement |
|---------|-----------|
| `crates/rr-core/src/crypto.rs` | Supprimer struct `CryptoProvider`, garder si tests les utilisent |
| `crates/rr-core/src/message.rs` | Struct → fonctions libres |
| `crates/rr-core/src/transport/mod.rs` | Supprimer trait, ré-export simple |
| `crates/rr-core/src/transport/nostr.rs` | Aucun (le struct reste) |
| `crates/rr-core/src/cell_transport.rs` | Supprimer legacy NIP-44 path, supprimer imports `CryptoProvider` |
| `crates/rr-core/src/cell.rs` | `cell_key_hex` → `#[serde(default)]` ou suppression |
| `crates/rr-core/src/lib.rs` | Ajuster les ré-exports |
| `crates/rr-cli/src/main.rs` | `MessageService::new().receive(...)` → `receive_message(...)` |
| `crates/rr-core/tests/cell_crypto.rs` | Adapter tests si `CryptoProvider` supprimé |

## Tests

- Les tests `cell_crypto.rs` (3 tests NIP-44) doivent continuer de passer — adapter pour appeler `nip44` directement ou via une fonction utilitaire.
- Les tests `cell_store.rs` — le champ `cell_key_hex` existe toujours, pas de changement.
- `crypto.rs` tests (NIP-44 roundtrip) — adapter si `CryptoProvider` supprimé.

## Critères de succès

1. `CryptoProvider` struct supprimé (fonctions remplacées par appels `nip44` directs)
2. `MessageService` struct supprimé (fonctions libres)
3. `TransportProvider` trait supprimé
4. Legacy NIP-44 path supprimé de `send_message()` et `listen()`
5. `cell_key_hex` rendu optionnel dans `Cell` (ou supprimé)
6. `cargo test --package rr-core` ✅
7. `cargo clippy --package rr-core -- -D warnings` ✅
8. `cargo build --package rr-cli` ✅
