# EPIC 3 — Sender Keys : Forward Secrecy & Key Rotation

**Status:** Draft  
**Date:** 2026-05-25  
**Depends on:** EPIC 2 (Groupes & Cellules)  

## Motivation

EPIC 2 introduced E2EE group messaging on Nostr via a static shared group key (NIP-44 self-DH). Trois faiblesses :

1. **Pas de forward secrecy** — une clé compromise expose tout l'historique
2. **Pas de rotation** — un membre sortant garde accès aux futurs messages
3. **`remove_member` cosmétique** — la clé partagée reste valide

EPIC 3 remplace la clé statique unique par **Sender Keys** (mécanisme Signal/WhatsApp pour les groupes) : chaque membre possède sa propre chaîne KDF dont on ratchette forward à chaque message. Sur `remove`/`rotate`, tous les membres regénèrent leur Sender Key et la redistribuent.

Pas de nouvelle dépendance Rust. La KDF chain utilise HKDF-sha256 (déjà disponible via nostr-rs).

## Principes

- **Pas de nouvelle dépendance** Rust ou externe
- **Pas de SQLite** — `cells.json` reste le seul store
- **`Cell` inchangé** — le `cell_key_hex` devient un champ `sender_keys: Vec<SenderKey>`
- **Même mécanisme de transport** que EPIC 2 : gift-wrap, rumor kind 13, tag `h`
- **0 migration** — les cellules existantes continuent de fonctionner

## Sender Key

```rust
pub struct SenderKey {
    pub member_pubkey: PublicKey,   // à qui appartient cette clé
    pub chain_key: [u8; 32],       // KDF chain key actuelle
    pub msg_count: u32,            // nombre de messages envoyés avec cette clé
    pub created_at_secs: u64,
}
```

Chaque `Cell` possède autant de `SenderKey` que de membres. Quand un membre émet, les autres activent la même ratchet pour déchiffrer.

### Ratchet

```
chain_key_i → HKDF(chain_key_i || "rr-sender-key") → message_key (32B) + chain_key_{i+1} (32B)

message_key_i → encrypt(msg)
chain_key_{i+1} stocké (prochain message)
chain_key_i supprimé (FS)
```

HKDF avec salt = `member_pubkey` (32B) et info = `"rr:group:sender_key:v1"`.

### Stockage dans cells.json

```json
{
    "id": "...",
    "label": "ma-cellule",
    "sender_keys": [
        {
            "member_pubkey": "d757c4e25...",
            "chain_key_hex": "ab34ff1...",
            "msg_count": 7,
            "created_at_secs": 1779657660
        }
    ],
    "members": [
        { "pubkey": "d757c4e25...", "label": "me" },
        { "pubkey": "f811a59c3...", "label": null }
    ],
    "created_at_secs": 1779657660
}
```

## CellTransport Changes

### send_message

1. Charge la `SenderKey` de l'émetteur dans la cellule
2. Ratchette : `chain_key_{n} → HKDF → message_key + chain_key_{n+1}`
3. Chiffre le rumor (kind 13, tag `h` = cell UUID) avec `message_key` via ChaCha20-Poly1305 ou AES-GCM
4. Stocke `chain_key_{n+1}` + `msg_count++`
5. Sauvegarde `cells.json`
6. Gift-wrap pour chaque membre (inchangé)

### listen (réception)

1. Reçoit un gift-wrap → unwrap → rumor kind 13
2. Trouve la cellule par tag `h`
3. Détermine l'émetteur via `sender_pk` (pubkey du wrapper gift-wrap)
4. Trouve la `SenderKey` de l'émetteur dans la cellule
5. Re-joue la ratchet : `chain_key_n → HKDF → message_key`
6. Déchiffre le rumor avec `message_key`
7. Stocke `chain_key_{n+1}` + avance `msg_count`
8. Sauvegarde `cells.json`

### create_cell

1. Génère une `SenderKey` pour soi-même : `chain_key = random_32_bytes()`, `msg_count = 0`
2. Gift-wrappe cette `SenderKey` à chaque autre membre via `send_cell_key`
3. Les autres membres stockent la `SenderKey` reçue dans leur `cells.json`

### invite_member (inchangé conceptuellement)

1. Génère une nouvelle `SenderKey` pour le nouveau membre
2. Lui envoie via gift-wrap
3. Les membres existants ajoutent une `SenderKey` vide pour le nouveau membre

### remove_member

1. Pour chaque membre RESTANT : génère une nouvelle `SenderKey` (chain_key aléatoire, msg_count = 0)
2. Pour chaque membre restant : gift-wrappe la Sender Key de chaque autre membre
3. Supprime le membre viré de `Cell.members` et de `Cell.sender_keys`
4. Sauvegarde `cells.json`
5. Le membre viré ne reçoit pas les nouvelles clés → ne peut plus déchiffrer

**Protocole de rotation :**
```
1. A initie remove(C)
2. A génère new_sk_A, new_sk_B (pour B)
3. A → B : gift-wrap(new_sk_A + new_sk_B) via existing chain
4. A applique new_sk_A + new_sk_B localement
5. B reçoit → applique new_sk_A + new_sk_B + supprime C
6. C n'a rien reçu → clés obsolètes
```

### rotate_key

```rust
pub async fn rotate_key(&self, cell_id: &str) -> Result<()>
```

Même logique que `remove_member` mais sans retirer personne. Tous les membres regénèrent leur Sender Key et re-distribuent. Utile pour PCS périodique.

## CLI Changes

Nouveau sous-commande :

```
rr group remove <CELL_ID> --member <NPUB>    # retirer un membre + rotation clés
```

`rr group send` et `rr group listen` restent inchangés (l'encryption change en interne).

## Forward Secrecy Guarantees

| Scénario | Avant EPIC 3 | Après EPIC 3 |
|----------|-------------|--------------|
| Membre viré lit les nouveaux messages | ✅ possible (même clé) | ❌ impossible (nouvelles clés) |
| Attaquant vole CellKey aujourd'hui | 🔓 tout l'historique | 🔒 messages avant dernière rotation protégés |
| Attaquant vole chain_key d'un membre | 🔓 tous ses messages | 🔒 messages AVANT sa chain_key actuelle protégés (ratchet KDF) |
| Rotation volontaire (`rr group rotate-key`) | n'existait pas | ✅ PCS : nouvel epoch, clés fraîches |

## Testing

| Test | Description |
|------|-------------|
| `sender_key_ratchet_forward` | chain_key_n → HKDF → message_key_n, vérifier chain_key_{n+1} ≠ chain_key_n |
| `sender_key_old_key_cannot_decrypt_new` | déchiffrer msg #5 avec chain_key_0 doit échouer (FS) |
| `remove_member_key_rotation` | remove → nouveaux chain_keys → l'ancien membre ne peut plus déchiffrer |
| `rotate_key_invalidates_old_keys` | rotate → anciens chain_keys invalides |
| `multiple_messages_in_order` | 5 messages émis, chaque récepteur ratchette correctement |
| `cell_transport_send_listen_roundtrip` | send → listen → plaintext match |

## Migration EPIC 2 → EPIC 3

- Les cellules EPIC 2 existantes (avec `cell_key_hex`) continuent de fonctionner
- `Cell` peut optionnellement être marqué `kind: SenderKeys | Static`
- Les nouvelles cellules utilisent Sender Keys
- Migration volontaire : `rr group rotate-key` → crée les Sender Keys, archive l'ancienne clé
- `listen` détecte le format du message (kind 13 encodé ChaCha20 vs NIP-44) → dispatch

## Implémentation

### Fichiers modifiés

- `crates/rr-core/src/cell.rs` — ajout `SenderKey`, `Cell.sender_keys`
- `crates/rr-core/src/cell_transport.rs` — implémentation Sender Key ratchet, send, listen, remove, rotate
- `crates/rr-cli/src/main.rs` — sous-commande `group remove` + `group rotate-key`

### Nouvelles fonctions

```rust
// cell.rs
pub struct SenderKey {
    pub member_pubkey: PublicKey,
    pub chain_key_hex: String,
    pub msg_count: u32,
    pub created_at_secs: u64,
}

// cell_transport.rs
fn ratchet_forward(chain_key: &[u8; 32]) -> ([u8; 32], [u8; 32])  // message_key + new_chain_key
pub async fn remove_member(&self, cell_id: &str, member: &PublicKey) -> Result<()>
pub async fn rotate_key(&self, cell_id: &str) -> Result<()>
```

## Sécurité

### Ratchet KDF

```
message_key_i = HKDF-SHA256(
    salt = member_pubkey,
    ikm = chain_key_i,
    info = "rr:group:sender_key:v1"
)
chain_key_{i+1} = first_32_bytes(HKDF-Expand(message_key_i, "rr:sender:next", 32))
```

La KDF chain est une fonction à sens unique : `chain_key_i` permet de dériver `chain_key_{i+1}` mais pas l'inverse.

### Pourquoi ChaCha20-Poly1305 au lieu de NIP-44 ?

Le message_key est une clé symétrique uniforme de 32B. NIP-44 ajoute une couche ECDH inutile quand on a déjà une clé symétrique. On utilise ChaCha20-Poly1305 directement (via `chacha20poly1305` crate, déjà dépendance transitive de nostr-rs), ou AES-256-GCM.

### Limitations

- Pas de PCS automatique : un attaquant qui vole une chain_key peut déchiffrer les messages du même membre jusqu'à la prochaine rotation (`rotate_key` ou `remove`)
- O(N) sur remove : parfait pour 3-5 membres, N operations de gift-wrap + N sauvegardes
- Concurrence : chaque membre écrit sa propre chain — Alice ratchette la clé d'Alice, Bob ratchette la clé de Bob. Aucune désynchronisation possible car chaque chain a un seul writer (le propriétaire).
