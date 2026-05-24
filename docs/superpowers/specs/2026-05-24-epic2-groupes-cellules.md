# EPIC 2 — Groupes & Cellules (Cell Encryption Groups)

## Goal

Permettre à un petit groupe de pairs (cellule, typiquement 3 membres, extensible à 5-10) de communiquer de façon chiffrée de bout en bout sur Nostr, sans qu'un observateur extérieur (relay compris) puisse lire le contenu ou déterminer la composition du groupe.

## Architecture

### Approche retenue : Clé de groupe partagée + Gift-wrap broadcast

- **Chiffrement** : NIP-44 (X25519 + ChaCha20 + HMAC-SHA256) avec une clé de groupe (`CellKey`) comme clé partagée
- **Transport** : kind 1059 gift-wrap (NIP-17), un rumor chiffré wrappé N fois (1 par destinataire + 1 pour soi-même)
- **Détermination du groupe** : tag `h` dans le rumor, contenant un UUID de cellule
- **Distribution de clé** : in-band via DM gift-wrap (la CellKey est chiffrée NIP-44 pour chaque membre et envoyée séparément)

### Pourquoi pas MLS / NIP-EE

Forward secrecy et ratchets complexes ne sont pas nécessaires pour des cellules de 3 membres stables qui communiquent sur des sessions de courte durée. La simplicité de NIP-44 + gift-wrap est suffisante et évite la complexité protocolaire de MLS.

### Métadonnées visibles

Le relay voit :
- Le `p` tag de chaque wrap (sait que X a envoyé un message à Y)
- Le kind 1059 (sait que c'est un gift wrap)
- Le timestamp

Le relay ne voit PAS :
- Le contenu déchiffré
- La clé de groupe
- La composition complète de la cellule (sauf par inférence des `p` tags de tous les membres)

Ce niveau de metadata leak est considéré acceptable pour les cellules de confiance.

## Data Model

### `CellMember`

| Champ | Type | Description |
|-------|------|-------------|
| `pubkey` | `PublicKey` | Clé publique Nostr du membre |
| `label` | `String` | Nom/label optionnel |
| `added_at` | `Timestamp` | Date d'ajout |

### `Cell`

| Champ | Type | Description |
|-------|------|-------------|
| `id` | `Uuid` | Identifiant unique de la cellule (utilisé comme `h` tag) |
| `label` | `String` | Nom de la cellule |
| `cell_key` | `SecretKey` | Clé X25519 partagée (stockée en hex localement) |
| `members` | `Vec<CellMember>` | Membres de la cellule |
| `created_at` | `Timestamp` | Date de création |

### Stockage

Fichier JSON unique `~/.config/reseau-racine/cells.json` :
- Tableau de `Cell`
- Chargé en mémoire au démarrage
- Sauvegardé après chaque mutation (create, invite)

Les messages déchiffrés ne sont **pas** persistés dans ce premier jet. Affichage console uniquement, re-fetch via le relay si nécessaire.

## Message Flow

### Création de cellule (`rr group create`)

1. Créateur génère une `CellKey` = `SecretKey::new()`
2. Crée un UUID v4 comme `cell_id`
3. Sauvegarde la `Cell` localement dans `cells.json`
4. Pour chaque membre (dont soi-même) :
   a. Chiffre la CellKey avec NIP-44 (clé créateur → clé publique du membre)
   b. Construit un rumor kind 13 avec : `content` = CellKey chiffrée, tags = `["h", cell_id]`
   c. Gift-wrap du rumor vers le membre
   d. Publie le wrap sur le relay

### Envoi de message (`rr group send`)

1. Charge la `Cell` locale (cell_key, membres, cell_id)
2. Chiffre le message avec NIP-44 (cell_key en tant que clé partagée)
   - Note : NIP-44 nécessite une paire (sk, pk). On utilise la cell_key comme sk du sender et la cell_key comme pk du receiver (même clé des deux côtés). NIP-44 spec supporte le cas `sk == shared_key, pk == shared_key` car c'est X25519 de base.
   - Vérification : `nip44::encrypt(shared_secret, shared_secret, content)` — test à écrire.
3. Construit un rumor kind 13 avec : `content` = message chiffré, tags = `["h", cell_id]`
4. Pour chaque membre :
   a. Gift-wrap du rumor (le même rumor) vers le membre
   b. Publie chaque wrap sur le relay

### Réception (`rr group listen`)

1. Subscribe à `Kind::GiftWrap` pour notre pubkey
2. Pour chaque event reçu :
   a. `client.unwrap_gift_wrap(event)` → rumor
   b. Si rumor.kind == 13 (ou kind réservé cellule) :
      - Extraire le tag `h` → `cell_id`
      - Chercher `cell_id` dans les cellules locales
      - Si trouvé : `nip44::decrypt(cell_key, cell_key, rumor.content)` → plaintext
      - Afficher : `[cell_label] sender: message`
      - Si pas trouvé : ignorer silencieusement

### Invitation (`rr group invite`)

1. Charge la `Cell` locale
2. Obtient la pubkey du nouveau membre
3. Chiffre la CellKey avec NIP-44 (clé créateur → clé publique nouveau membre)
4. Envoie via gift-wrap
5. Ajoute le membre à `Cell.members`
6. Sauvegarde `cells.json`

## CLI Commands

```
rr group create --label "famille" --members <npub1,npub2>
rr group list
rr group info <cell-id>
rr group invite <cell-id> --member <npub>
rr group send <cell-id> --message "texte"
rr group listen <cell-id>
```

Toutes les commandes `group` utilisent l'identité courante (mêmes `IdentityManager` et `KeySource` que les commandes existantes).

## Implementation Plan

### Modules à créer/modifier

1. **`crates/rr-core/src/cell.rs`** (nouveau)
   - Types : `Cell`, `CellMember`, `CellStore`
   - `CellStore::load()`, `CellStore::save()`, `CellStore::find(id)`
   - Sérialisation JSON via serde

2. **`crates/rr-core/src/cell_transport.rs`** (nouveau)
   - `CellTransport` struct avec client relay + cell_store
   - `create_cell(label, members) -> Cell`
   - `send_message(cell_id, content)`
   - `listen(cell_id) -> impl Stream<Item = CellMessage>`
   - `invite_member(cell_id, pubkey)`

3. **`crates/rr-core/src/lib.rs`**
   - Ajouter `pub mod cell;`
   - Ajouter `pub mod cell_transport;`

4. **`crates/rr-cli/src/main.rs`**
   - Ajouter `Group` subcommand à l'enum `Commands`
   - Ajouter `Group { command: GroupCommands }` avec `#[command(subcommand)]`
   - Implémenter les handlers : `cmd_group_create`, `cmd_group_list`, `cmd_group_info`, `cmd_group_invite`, `cmd_group_send`, `cmd_group_listen`

### Test plan

1. Unit tests pour `CellStore` (load/save/find d'une cellule)
2. Unit tests pour le chiffrement avec shared key symmetric (`nip44::encrypt(shared_key, shared_key, content)`)
3. Test d'intégration (si relay disponible) : create → send → listen

### Non-goals (v1)

- Pas de rotation de clé
- Pas de stockage persistant des messages
- Pas de gestion des départs de membre
- Pas de MLS / forward secrecy
- Pas de notifications push
- Pas de support Tauri (console uniquement)

## Self-Review

- **Placeholders** : aucun
- **Contradictions** : aucune détectée
- **Scope** : focussed, un seul sous-système (groupes)
- **Ambiguïtés** :
  - Le type de tag `h` et le kind exact pour le rumor cellule sont à confirmer. NIP-29 utilise kind `kind: 1111` pour `h` tag groups. On pourrait utiliser kind 13 (plaintext note) ou un custom kind. → Décision utilise kind 13 avec tag `h` pour la simplicité, le client filtre par tag côté réception. Pas besoin d'un kind spécial.
  - `nip44::encrypt(shared_key, shared_key, content)` — cela fonctionne-t-il techniquement ? NIP-44 utilise `x25519_diffie_hellman(secret, public)` qui fonctionne avec `x25519(shared, shared)` → donne une clé partagée déterministe. C'est l'équivalent d'une clé symétrique via X25519. C'est sûr car seul le groupe possède la CellKey.
- **Divergences design/implémentation** : aucune
