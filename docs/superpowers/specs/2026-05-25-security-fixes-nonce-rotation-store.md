# EPIC SEC-1 — Security fixes: nonce atomicity, authenticated rotation, atomic store

## Priority

P0 — Bloquant pour toute utilisation réelle.

## Problèmes

Trois failles identifiées par l'audit crypto (cf. `docs/superpowers/specs/2026-05-25-epic3-sender-keys-rotation.md` pour le contexte Sender Key) :

### 1. Nonce ChaCha20 à zéro + ratchet non atomique → réutilisation de clé

**Fichier :** `crates/rr-core/src/sender_key.rs`, `crates/rr-core/src/cell_transport.rs`

**Problème :** `sender_key.rs:31` utilise `Nonce::default()` (zéro). La sécurité repose sur le fait que chaque message utilise une `message_key` unique via la ratchet HKDF. Mais dans `cell_transport.rs` l'ordre est :

1. Lire `chain_key` du store
2. `ratchet_forward()` → `message_key`
3. Chiffrer avec `message_key`
4. Envoyer les messages (appels réseau — peut planter)
5. *Puis* mettre à jour `chain_key` dans le store

Si le processus plante entre 3 et 5, la même `chain_key` est réutilisée au redémarrage → **même `message_key`** → **même nonce** → ChaCha20 keystream identique → XOR des ciphertexts = XOR des plaintexts. Confidentialité détruite.

**Solution choisie :** Deux changements combinés :

(a) Inclure `msg_count` dans l'info string HKDF — lie la clé dérivée au compteur de message.
(b) **Réordonner le save AVANT le réseau** — dans `send_message()`, le store est mis à jour (chain_key_hex + msg_count) et sauvegardé AVANT d'envoyer les gift-wraps. Si le processus crashe pendant l'envoi, le msg_count est déjà consommé → la prochaine tentative utilise un msg_count différent → clé différente.

**Risque :** Un crash pendant l'envoi brûle 1 slot msg_count (sur 2^64). Acceptable comparé à une fuite de confidentialité.

**Détail technique :**
```rust
// Avant (vulnérable) :
let (msg_key, next_chain) = ratchet_forward(&chain);
encrypt + send...  // peut planter ici
save store...      // jamais atteint si crash

// Après (sûr) :
let (msg_key, next_chain) = ratchet_forward(&chain, sk.msg_count);
save store (chain_key = next_chain, msg_count += 1)  // AVANT send
encrypt + send...
```

**Signature ratchet_forward :** `fn ratchet_forward(chain_key: &[u8; 32], msg_count: u64) -> ([u8; 32], [u8; 32])` — ajout du paramètre `msg_count`. 4 tests à mettre à jour.

### 2. `handle_key_rotation()` non authentifiée → DoS

**Fichier :** `crates/rr-core/src/cell_transport.rs` (méthode privée)

**Problème :** N'importe quel pair Nostr peut gift-wraper `{"action": "key_rotation", "sender_keys": [...]}` vers la pubkey d'un membre. La fonction ne vérifie PAS que l'expéditeur est membre de la cellule. Les sender_keys sont remplacées/écrasées → messages futurs indéchiffrables.

**Solution :** Passer `sender_pk: &PublicKey` à `handle_key_rotation()`. Vérifier que `sender_pk ∈ cell.members` avant d'appliquer la rotation. Si l'expéditeur n'est pas membre, logger un warning et ignorer.

**Choses à creuser :**
- `sender_pk` est disponible dans la closure `handle_notifications` (ligne ~560) — déjà extrait via `let sender_pk = unwrapped.sender;`
- Que faire si le membre retiré n'est plus dans `cell.members` mais était légitime avant ? La rotation est envoyée aux *membres restants*. Si l'ancien membre la reçoit (parce qu'il écoutait encore), il ne doit pas l'appliquer — c'est correct, il n'est plus dans `cell.members`.
- Logger via `eprintln!` (style existant du projet) si un non-membre tente une rotation, pour détection d'attaque.
- Ajouter un test unitaire : envoyer une `key_rotation` d'un non-membre → l'opération est ignorée, store inchangé.

### 3. `CellStore::save()` non atomique + corruption silencieuse

**Fichier :** `crates/rr-core/src/cell.rs`

**Problème :**
- `save()` : `std::fs::write(path, json)` — crash pendant l'écriture → fichier tronqué.
- `load()` : `.ok().and_then(parse).unwrap_or_default()` — fichier corrompu → store vide, zéro warning.

**Solution :**
- `save()` : écrire dans `path.tmp`, puis `fs::rename(path.tmp, path)`. Le rename est atomique sur un même filesystem (POSIX).
- `load()` : si un fichier `.tmp` existe (crash précédent), le supprimer. Si `cells.json` existe mais ne parse pas, logger `eprintln!` AVEC le message d'erreur. Retourner `CellStore::default()` (graceful degradation).
- Logger les erreurs via `eprintln!` (pas de dépendance `log`).

## Fichiers impactés

| Fichier | Changement |
|---------|-----------|
| `crates/rr-core/src/sender_key.rs` | Ajout `msg_count` param, mise à jour info string HKDF |
| `crates/rr-core/src/cell_transport.rs` | Ordre atomique ratchet, `sender_pk` check dans handle_key_rotation |
| `crates/rr-core/src/cell.rs` | `save()` atomique, `load()` log erreur |
| `crates/rr-core/tests/sender_key.rs` | Mise à jour 4 tests (nouveau param `msg_count`) |

## Tests

- `sender_key::ratchet_forward()` avec `msg_count=0` et `msg_count=1` produisent des clés différentes (même `chain_key`)
- `handle_key_rotation()` depuis un non-membre → store non modifié, eprintln! warning
- `CellStore::save()` → fichier existe, contient JSON valide (vérifier via load)
- `CellStore::load()` fichier corrompu → log erreur, retourne store vide (comportement actuel)

## Critères de succès

1. `ratchet_forward(chain_key, 0) != ratchet_forward(chain_key, 1)` (clés différentes avec msg_count différent)
2. `handle_key_rotation` depuis non-membre → store inchangé
3. Crash pendant `save()` ne corrompt pas `cells.json`
4. `load()` sur fichier corrompu → log + default, pas de panic
5. `cargo test --package rr-core` ✅
6. `cargo clippy --package rr-core -- -D warnings` ✅

## Dépendances

Aucune. Travaille dans le code existant sans nouveau module. Pas de dépendance `log` — utilisation de `eprintln!` (style existant).
