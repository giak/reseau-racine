# EPIC TEST-1 — CellTransport unit tests

## Priorité

P1 — Le module le plus complexe du projet (700+ lignes) a 0 tests non ignorés.

## Problème

`crates/rr-core/src/cell_transport.rs` est le cœur du protocole de groupe : création, invitation, envoi, écoute, retrait, rotation de clés. Il contient 7 méthodes publiques dont la fonction `listen()` (264 lignes). Il a exactement 1 test dans `tests/cell_transport.rs` marqué `#[ignore]` (besoin d'un relay Nostr).

Les tests d'intégration avec un vrai relay sont utiles mais :
- Fragiles (dépendent de l'état du relay)
- Lents (connexion WebSocket, attente)
- Impossible en CI sans service dédié
- Ne testent pas les chemins d'erreur (corruption, désynchronisation, timeouts)

## Approche

Deux stratégies complémentaires :

### A. Extraire les helpers testables (recommandé)

Extraire la logique métier des grosses méthodes pour créer des fonctions pures et testables, SANS dépendance réseau.

Ce qu'on extrait :

1. **`try_decrypt_sender_key(ciphertext, chain_key_hex, sender_pk) -> Result<(String, [u8;32]), Error>`**
   - Prend le ciphertext + chain_key hex + sender_pk
   - Exécute ratchet_forward + decrypt_with_message_key
   - Retourne (plaintext, next_chain_key) ou erreur
   - Testable sans réseau, sans store
   - Remplace les 3 blocs dupliqués dans `listen()`

2. **`try_decrypt_legacy(rumor_content, cell_key_hex) -> Option<String>`**
   - Prend le content + cell_key_hex (legacy NIP-44)
   - Retourne plaintext ou None
   - Teste le legacy path isolément

3. **`update_sender_key_after_decrypt(store, cell_id, sender_pk, next_chain_key)`**
   - Met à jour chain_key_hex + incrémente msg_count dans le store
   - Testable avec un CellStore in-memory (pas de filesystem)

4. **`handle_key_rotation`** — déjà extraite, mais améliorée avec check `sender_pk`

### B. Mock léger du store

`CellTransport` prend `Arc<Mutex<CellStore>>`. En test, on peut construire un `CellTransport` avec un store in-memory (pas de fichier). Les méthodes qui n'ont pas besoin de réseau (`remove_member` après `send_cell_key`, `rotate_key` après distribution) deviennent testables :

```rust
#[cfg(test)]
impl CellTransport {
    pub fn new_for_test(identity: Identity, config: Config, transport: NostrTransport) -> Self {
        let store = Arc::new(tokio::sync::Mutex::new(CellStore::default()));
        Self { keys: identity.keys, client: transport.client().clone(), store }
    }
}
```

Ce qu'on peut tester avec ça :
- `remove_member` modifie bien le store (membres, sender_keys)
- `rotate_key` regénère bien les sender_keys
- `remove_member` sur cellule inexistante → erreur
- `remove_member` par un non-membre → erreur

Ce qu'on NE peut PAS tester :
- L'envoi réseau (gift-wrap) — nécessite mock ou relay
- `listen()` en mode réel — nécessite events entrants
- `send_message` — dépend du réseau

### C. Tests pour listen() via construction d'événements

Pour tester `listen()` en isolation, on peut construire des `Event` Nostr artificiels et les injecter via `RelayPoolNotification::Event`. Mais la méthode utilise `client.unwrap_gift_wrap()` qui nécessite un vrai client Nostr. Solution : extraire le handler de notification (`on_event(event, ...)`) en fonction séparée, testable avec des événements pré-construits.

```rust
// Extrait :
pub async fn on_cell_event(
    event: &Event,
    sender_pk: PublicKey,
    rumor: &UnsignedEvent,
    store_arc: &Arc<Mutex<CellStore>>,
    target_cell_id: &Option<String>,
) -> Result<(), CellError> {
    // ... toute la logique de routage + decrypt + display
}
```

## Fichiers impactés

| Fichier | Changement |
|---------|-----------|
| `crates/rr-core/src/cell_transport.rs` | Extraction helpers, `on_cell_event()`, `try_decrypt_sender_key()`, `try_decrypt_legacy()` |
| `crates/rr-core/tests/cell_transport.rs` | 10-15 nouveaux tests (helpers + store) |
| `crates/rr-core/src/lib.rs` | Si nouveaux types exportés |

## Tests à écrire (par ordre de priorité)

### Helpers purs (6 tests)

1. `try_decrypt_sender_key` avec ciphertext valide → retourne plaintext + next_chain
2. `try_decrypt_sender_key` avec mauvais chain_key → erreur
3. `try_decrypt_sender_key` avec ciphertext corrompu → erreur
4. `install_sender_key` avec clé valide → ajoute/remplace dans SenderKey[]
5. `handle_key_rotation` de membre légitime → sender_keys mises à jour
6. `handle_key_rotation` de non-membre → store inchangé

### CellTransport store (4 tests)

7. `remove_member` en isolation (mock store, sans réseau) → membres filtrés
8. `remove_member` self (tenter de se retirer soi-même) → erreur
9. `remove_member` cellule inexistante → erreur
10. `rotate_key` → toutes les sender_keys ont msg_count=0, chain_key_hex changé

### Intégration (2 tests)

11. `send_cell_key` VIA CONSTRUCTED EVENTS → gift-wrap envoyé (vérifier via subscribe)
12. Décrypt roundtrip avec helpers extraits (simule le flux complet sans réseau)

### Regression (3 tests)

13. Legacy NIP-44 decrypt roundtrip (inchangé ? à vérifier)
14. Empty cell_key_hex → legacy skip (pas de panic, pas d'erreur)
15. Sender key manquant → legacy fallback tente NIP-44

## Critères de succès

1. Au moins 10 nouveaux tests non ignorés qui passent
2. `CellTransport` n'a plus de dépendance réseau obligatoire pour les tests de store
3. `cargo test --package rr-core` ✅ (51 → ~65 tests)
4. `cargo clippy --package rr-core -- -D warnings` ✅
5. Les helpers extraits sont documentés (docstrings)

## Non-goals

- Mock du réseau Nostr complet (trop coûteux pour le bénéfice)
- Test de `listen()` avec vrai relay en CI (pas fiable)
- Couverture à 100% de `cell_transport.rs` (irréaliste sans mock complexe)

## Dépendances

- `CellTransport::new_for_test()` — méthode `#[cfg(test)]` seulement
- Pas de nouvelles dépendances Cargo
