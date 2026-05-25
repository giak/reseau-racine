# EPIC REFACTOR-1 — Refactor listen() decrypt duplication

## Priorité

P2 — Duplication = 2x risque de bug, 3x coût de maintenance.

## Problème

`CellTransport::listen()` (264 lignes, `cell_transport.rs:414-678`) contient **3 blocs quasi-identiques** de logique de déchiffrement Sender Key :

1. **Mode spécifique (cell_id donné)** — lignes ~479-523
2. **Mode découverte cellule connue** — lignes ~593-648
3. **Pattern similaire dans send_message** — lignes ~173-210 (post-envoi ratchet)

Chaque bloc fait exactement la même chose :
1. Trouver la `SenderKey` pour `sender_pk` dans `cell.sender_keys`
2. `hex::decode_to_slice(chain_key_hex, &mut chain)`
3. `ratchet_forward(&chain)` → `(msg_key, next_chain)`
4. `base64::decode(rumor.content)` → `cipher_bytes`
5. `decrypt_with_message_key(&msg_key, &cipher_bytes)` → `plaintext`
6. Mettre à jour `chain_key_hex` et `msg_count` dans le store
7. Afficher le message

85+ lignes copiées-collées. Si on modifie un bloc (ex: ajouter AD), les autres deviennent incohérents.

## Solution

### 1. Extraire `try_decrypt_sender_key_event()`

Fonction libre qui prend les entrées brutes et retourne le résultat + next_chain :

```rust
// Dans cell_transport.rs (ou nouveau fichier cell_decrypt.rs)
pub fn try_decrypt_sender_key_event(
    sender_pk: &PublicKey,
    rumor_content: &str,
    sender_keys: &[SenderKey],
) -> Result<(String, [u8; 32]), CellError> {
    let sk = sender_keys
        .iter()
        .find(|sk| sk.member_pubkey == *sender_pk)
        .ok_or(CellError::SenderKeyDecrypt("aucune clé pour cet expéditeur".into()))?;

    let mut chain = [0u8; 32];
    hex::decode_to_slice(&sk.chain_key_hex, &mut chain)
        .map_err(|e| CellError::SenderKeyDecrypt(format!("hex invalide: {e}")))?;
    let (msg_key, next_chain) = sender_key::ratchet_forward(&chain, sk.msg_count);
    let engine = base64::engine::general_purpose::STANDARD;
    let cipher_bytes = engine
        .decode(rumor_content)
        .map_err(|e| CellError::SenderKeyDecrypt(format!("base64 invalide: {e}")))?;
    let plaintext = sender_key::decrypt_with_message_key(&msg_key, &cipher_bytes)
        .map_err(|e| CellError::SenderKeyDecrypt(format!("déchiffrement échoué: {e}")))?;

    Ok((plaintext, next_chain))
}
```

**Testable sans réseau, sans store, sans closure async.** Prend des slices, retourne `Result`.

### 2. Extraire `display_cell_message()`

```rust
fn display_cell_message(cell_label: &str, sender_pk: &PublicKey, plaintext: &str) {
    if sender_pk != self_pk {
        let snpub = sender_pk.to_bech32().unwrap_or_else(|_| sender_pk.to_string());
        println!("[{}] {}: {}", cell_label, snpub, plaintext);
    }
}
```

### 3. Extraire `on_cell_event()`

```rust
// Gère tout le routage d'un événement reçu
pub async fn on_cell_event(
    store_arc: &Arc<Mutex<CellStore>>,
    my_pk: &PublicKey,
    sender_pk: &PublicKey,
    rumor: &UnsignedEvent,
    target_cell_id: &Option<String>,
) {
    let h_tag_val = rumor.tags.iter()
        .find(|t| t.kind() == TagKind::Custom("h".to_string().into()))
        .and_then(|t| t.content())
        .map(|s| s.to_string());

    let h_tag = match h_tag_val {
        Some(v) => v,
        None => return,
    };

    // Mode spécifique: filtrer par cell_id
    if let Some(tid) = target_cell_id {
        if &h_tag != tid { return; }
        // Récupérer cellule, try_decrypt_sender_key_event, display
    } else {
        // Mode découverte: key_rotation check, puis decrypt
    }
}
```

### 4. `listen()` devient :

```rust
pub async fn listen(&self, cell_id: Option<&Uuid>) -> Result<(), CellError> {
    // Setup: filtre, subscribe
    // handle_notifications(|notification| {
    //     unwrap → on_cell_event(...)
    // })
}
```

On passe de 264 lignes à ~50, le reste est délégué.

## Résultat attendu

| Mesure | Avant | Après |
|--------|-------|-------|
| Lignes de `listen()` | 264 | ~50 |
| Blocs décrypt dupliqués | 3 | 0 |
| Fonctions extraites | 0 | 2 (`try_decrypt_sender_key_event`, `on_cell_event`) |
| Testable sans réseau | non | oui (`try_decrypt_sender_key_event`) |

## Attention

- `on_cell_event` doit gérer les mêmes chemins que `listen()` actuellement (key_rotation, legacy, découverte, etc.)
- La fermeture async capte `store_arc`, `keys`, `client` — `on_cell_event` prend `store_arc` et `my_pk` directement
- `display_message` utilise le `my_pk` pour filtrer ses propres messages — à passer explicitement
- Ne pas toucher à `send_message` dans ce refactor (sauf si le bloc décrypt est aussi extrait et réutilisé)

## Fichiers impactés

| Fichier | Changement |
|---------|-----------|
| `crates/rr-core/src/cell_transport.rs` | Extraire `try_decrypt_sender_key_event`, `on_cell_event`, `display_cell_message` |
| `crates/rr-core/src/lib.rs` | Si nouvelles fonctions exportées |
| `crates/rr-core/tests/cell_transport.rs` | Tests pour `try_decrypt_sender_key_event` |

## Solutions alternatives rejetées

- **Macros** : résoud la duplication mais rend le code moins lisible (debug).
- **Fonction inline avec closure** : les 3 blocs ont assez de différences (un lock store, pas de lock, display différent) pour qu'une seule fonction paramétrée soit artificielle. `try_decrypt_sender_key_event` capture la partie commune (décrypt), `on_cell_event` capture le routage — c'est la bonne granularité.

## Critères de succès

1. Plus aucun bloc décrypt Sender Key dupliqué
2. `try_decrypt_sender_key_event` testable (tests unitaires)
3. `listen()` passe de 264 à ~50 lignes
4. `cargo test --package rr-core` ✅
5. `cargo clippy --package rr-core -- -D warnings` ✅
6. `cargo build --package rr-cli` ✅
