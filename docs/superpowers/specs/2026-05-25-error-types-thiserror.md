# EPIC ERR-1 — Typed errors with thiserror

## Priorité

P2 — Nuit à la maintenabilité et au diagnostic, mais pas de faille de sécurité directe.

## Problème

~15 fonctions dans `rr-core` retournent `Result<_, Box<dyn std::error::Error>>`. Les erreurs sont créées via `format!("...")` — des `String` sans type. L'appelant ne peut pas distinguer :

- "timeout réseau" de "signature invalide" de "cellule introuvable"
- Faire du pattern matching pour décider du retry vs abort
- Propager l'erreur avec contexte

Déjà un symptôme en production : `rr-stress:129` fait `msg.contains("timeout")`.

## Solution

Ajouter `thiserror = "2"` dans `rr-core/Cargo.toml`. Définir des enums d'erreur par module.

### Architecture des erreurs

**Principe :** Une erreur `CellError` pour le module cell_transport, une `IdentityError` pour identity, etc. Pas d'énum géante — chaque module a son type.

```rust
// crates/rr-core/src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CellError {
    #[error("Cellule {0} introuvable")]
    NotFound(String),

    #[error("Vous n'êtes pas membre de cette cellule")]
    NotMember,

    #[error("Échec déchiffrement Sender Key: {0}")]
    SenderKeyDecrypt(String),

    #[error("Échec déchiffrement legacy: {0}")]
    LegacyDecrypt(String),

    #[error("Rotation de clés rejetée: {0}")]
    KeyRotationRejected(String),

    #[error("Erreur store: {0}")]
    Store(#[from] std::io::Error),

    #[error("Erreur réseau: {0}")]
    Transport(String),
}

#[derive(Error, Debug)]
pub enum IdentityError {
    #[error("Identité introuvable: {0}")]
    NotFound(String),

    #[error("Clé invalide: {0}")]
    InvalidKey(String),

    #[error("Erreur fichier: {0}")]
    Io(#[from] std::io::Error),

    #[error("Erreur seed phrase: {0}")]
    SeedPhrase(String),
}
```

### Migration

Remplacer graduellement `Result<_, Box<dyn Error>>` par les types spécialisés :

| Fonction | Ancien type | Nouveau type |
|----------|------------|--------------|
| `CellTransport::create_cell` | `Box<dyn Error>` | `CellError` |
| `CellTransport::invite_member` | `Box<dyn Error>` | `CellError` |
| `CellTransport::send_message` | `Box<dyn Error>` | `CellError` |
| `CellTransport::remove_member` | `Box<dyn Error>` | `CellError` |
| `CellTransport::rotate_key` | `Box<dyn Error>` | `CellError` |
| `CellTransport::listen` | `Box<dyn Error>` | `CellError` |
| `CellTransport::handle_key_rotation` | `Box<dyn Error>` | supprimé (devient async fn retourne bool) |
| `CellTransport::send_cell_key` | `Box<dyn Error>` | `CellError` |
| `IdentityManager::load` | `Box<dyn Error>` | `IdentityError` |
| `IdentityManager::save` | `Box<dyn Error>` | `IdentityError` |
| `IdentityManager::from_nsec` | `Box<dyn Error>` | `IdentityError` |
| `IdentityManager::from_seed_phrase` | `Box<dyn Error>` | `IdentityError` |

### `From` implémentations

- `From<nostr::nostr::key::Error>` pour `IdentityError`
- `From<serde_json::Error>` pour `CellError` (ou `IdentityError` selon contexte)
- `From<nostr_sdk::client::Error>` pour `CellError`
- `From<nip44::Error>` pour `CellError`

### CLI adaptation

Dans `main.rs`, chaque handler CLI a déjà `match result { Ok(_) => ..., Err(e) => eprintln!("❌ Erreur: {}", e) }`. Le `Display` implémenté par `thiserror` donne directement un message lisible — aucun changement nécessaire dans le CLI.

Pour le diagnostic avancé, `{:#}` (alternate Display) peut afficher les sources chaînées.

### `anyhow` ?

Ne pas utiliser `anyhow` pour l'instant. `thiserror` donne des types précis pour l'API publique. Si le besoin d'erreur ad-hoc apparaît dans les handlers CLI, on ajoutera `anyhow` plus tard. YAGNI.

## Fichiers impactés

| Fichier | Changement |
|---------|-----------|
| `crates/rr-core/Cargo.toml` | Ajout `thiserror = "2"` |
| `crates/rr-core/src/error.rs` | Nouveau : `CellError`, `IdentityError` |
| `crates/rr-core/src/lib.rs` | `pub mod error;` + re-exports |
| `crates/rr-core/src/cell_transport.rs` | Migrer 7+ méthodes vers `CellError` |
| `crates/rr-core/src/identity.rs` | Migrer 5+ méthodes vers `IdentityError` |
| `crates/rr-core/src/crypto.rs` | Déjà bon (`Result<_, nip44::Error>`) |
| `crates/rr-core/src/message.rs` | Migrer vers `CellError` si conservé |
| `crates/rr-core/src/cell.rs` | `CellStore::save()` vers `CellError` |
| `crates/rr-cli/src/main.rs` | Possible ajustement mineur des messages d'erreur |

## Tests

- Les tests existants qui checkent `is_err()` continuent de marcher (le type d'erreur change mais `Result::is_err()` reste).
- Ajouter un test de type : `fn assert_cell_error_send_sync() where CellError: Send + Sync {}`
- Vérifier que `format!("{}", CellError::NotFound("x".into()))` donne "Cellule x introuvable"

## Critères de succès

1. `thiserror` dans les dépendances
2. `error.rs` avec `CellError` et `IdentityError`
3. Toutes les méthodes `cell_transport.rs` retournent `Result<_, CellError>` (sauf `handle_key_rotation`)
4. Toutes les méthodes `identity.rs` retournent `Result<_, IdentityError>`
5. `cargo test --package rr-core` ✅
6. `cargo clippy --package rr-core -- -D warnings` ✅
7. `cargo build --package rr-cli` ✅

## Pièges à éviter

- **Ne pas rendre les enums trop grands** : si `CellError` dépasse 8 variantes, scinder. Pour l'instant ~7 variantes c'est parfait.
- **Ne pas implémenter `From` pour tout** : `From<nostr_sdk::Error>` peut masquer des erreurs inattendues. Préférer `map_err(|e| CellError::Transport(e.to_string()))` pour les cas où le type source n'est pas critique.
- **Ne pas casser l'API des tests** : les tests existants utilisent `is_err()` et `is_ok()` — insensible au type d'erreur.
