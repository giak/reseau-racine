# EPIC — POC Fil Rouge : "Premier Message Chiffré"

> **Objectif** : prouver que l'identité cryptographique + chiffrement E2E via Nostr fonctionne. Rien d'autre.

---

## Architecture du POC

```
┌─────────────────────────────────────────────────────────────────────┐
│                        ALICE (Client Rust CLI)                      │
│                                                                     │
│  npub1alice... (secp256k1)                                         │
│                                                                     │
│  1. "rr send bob Salut, ça marche"                                 │
│  2. Crée un Rumor (kind 14) — message en clair                     │
│  3. Signe avec clé utilisateur → Seal (kind 13)                     │
│  4. Chiffre avec NIP-44 V2 (ChaCha20-Poly1305) → GiftWrap (1059)   │
│  5. Publie sur relais Nostr (WebSocket)                            │
│                                                                     │
│  tout ça via une seule appel : nostr-sdk::Client::send_private_msg │
└──────────────────────────────┬──────────────────────────────────────┘
                               │
                    WebSocket wss://relay.damus.io
                    EVENT ["", {kind:1059, ...}]
                               │
┌──────────────────────────────▼──────────────────────────────────────┐
│                        RELAIS NOSTR PUBLIC                          │
│                                                                     │
│  Voit seulement :                                                   │
│  - kind: 1059 (GiftWrap)                                           │
│  - pubkey: <clé éphémère>                                          │
│  - tags: [["p", "npub_bob"]]                                       │
│  - content: <blob chiffré NIP-44>                                  │
│                                                                     │
│  Le relais NE SAIT PAS :                                           │
│  - Qui est l'expéditeur réel                                       │
│  - Quel est le contenu                                             │
│  - Que c'est un message (pas un autre type d'événement)            │
└──────────────────────────────┬──────────────────────────────────────┘
                               │
                    WebSocket subscription
                    REQ ["", {kinds:[1059], authors:[npub_bob]}]
                               │
┌──────────────────────────────▼──────────────────────────────────────┐
│                        BOB (Client Rust CLI)                        │
│                                                                     │
│  npub1bob... (secp256k1)                                           │
│                                                                     │
│  1. "rr sync"                                                      │
│  2. Subscribe aux events kind 1059 pour sa pubkey                  │
│  3. Reçoit GiftWrap → déchiffre avec client.unwrap_gift_wrap()     │
│  4. Extrait sender + rumor (UnsignedEvent)                         │
│  5. Affiche : "Message de alice: Salut, ça marche"                 │
│     ✓ Signature vérifiée (secp256k1 dans le seal)                  │
│     ✓ Déchiffrement réussi (ChaCha20-Poly1305 AEAD)                │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Code actuel — Audit complet (2026-05-23)

### Ce qui existe déjà (Phase 0 + Phase 1)
| Module | Statut | Tests | Fichiers |
|--------|--------|-------|----------|
| **Identity** | ✅ COMPLET | 18 tests | `crates/rr-core/src/identity.rs` |
| **Crypto** | ✅ COMPLET + 12 tests + 3 proptest | 15 | `crates/rr-core/src/crypto.rs` |
| **Message** | 🟡 STUB | 0 | `crates/rr-core/src/message.rs` |
| **Transport** | 🟡 BASIC | 0 | `crates/rr-core/src/transport/*.rs` |
| **CLI** | 🟡 PARTIAL | — | `crates/rr-cli/src/main.rs` |

### Détail du code existant

**Identity (`identity.rs`)** : création/sauvegarde/chargement clés, seed phrase BIP39 (12 mots), nsec/npub bech32. Stockage JSON `~/.rr/keys.json` (permissions 0600). Manager avec `load_or_create()`. 18 tests unitaires passent.

**Crypto (`crypto.rs`)** : wrapper autour de `nostr::nips::nip44::{encrypt, decrypt}`. Prend `&SecretKey`, `&PublicKey`, `&str`. Rejette messages vides et > 65535 bytes. 12 tests unitaires + 3 proptest passent.

**Message (`message.rs`)** : STUB. Deux fonctions qui wrappent l'API nostr-sdk :
- `send(client, receiver_pubkey, content)` → `client.send_private_msg(receiver, content, [])`
- `receive(client, gift_wrap)` → `client.unwrap_gift_wrap(gift_wrap)`
- **Problème** : prend un `&Client` déjà connecté — ne gère pas la création du transport avec les vraies clés.

**Transport (`transport/`)** : `NostrTransport` encapsule `nostr-sdk::Client`. Deux constructeurs :
- `new(relay_url)` → génère des clés éphémères (⚠️ inutilisable pour NIP-17)
- `with_keys(relay_url, keys)` → utilise des clés données (✅ nécessaire pour NIP-17)
- **Problème** : `NostrTransport::new()` est le constructeur par défaut mais il est inutilisable pour envoyer des messages (clés éphémères).

**CLI (`main.rs`)** : 
| Commande | Statut |
|----------|--------|
| `rr init` | ✅ Génère identité + seed phrase + sauvegarde |
| `rr identity` | ✅ Affiche npub |
| `rr add-contact <npub> <name>` | ✅ Stocke dans `contacts.json` |
| `rr contacts` | ✅ Liste les contacts |
| `rr restore <phrase>` | ✅ Restaure depuis seed phrase |
| `rr send <contact> <message>` | 🔜 STUB |
| `rr sync` | 🔜 STUB |

---

## Stack

| Composant | Choix | Version | Pourquoi |
|-----------|-------|---------|----------|
| **Langage** | Rust | stable 1.95.0 | Pure Rust, binaire unique, sécurité mémoire |
| **Crypto** | `nostr` crate | 0.44.3 | NIP-44 ChaCha20-Poly1305 + NIP-59 seal/giftwrap natif |
| **SDK Nostr** | `nostr-sdk` crate | 0.44.1 | Client WebSocket, `send_private_msg`, `unwrap_gift_wrap` |
| **Transport** | WebSocket (tokio-tungstenite) | via nostr-sdk | Async, multiplexé |
| **Interface** | CLI (clap) | `rr {init,identity,send,sync,...}` | Pas de frontend à build, test rapide |
| **Stockage** | Fichier JSON local | — | `~/.rr/keys.json`, `~/.rr/contacts.json` |
| **Async** | tokio | 1 (features=full) | Runtime standard |
| **CLI framework** | clap | 4.6 | Parse, Subcommand derive |

### Pourquoi NIP-17 (pas NIP-04)
NIP-04 est déprécié : AES-256-CBC sans AEAD, fuites métadonnées, **CVE-2026-41301**. NIP-17 via NIP-44 + NIP-59 est le standard actuel, supporté nativement par les crates 0.44.

### API utilisée

**Envoi** — une seule ligne grâce à l'abstraction `nostr-sdk` :
```rust
// Cette appel fait : rumor (kind 14) → seal (kind 13) → gift wrap (1059)
// signé avec les clés du signer, chiffré avec NIP-44 pour receiver
client.send_private_msg(receiver_pubkey, "Salut, ça marche", vec![]).await?;
```

**Réception** — unwrap automatique :
```rust
let unwrapped: UnwrappedGift = client.unwrap_gift_wrap(&event).await?;
// unwrapped.sender → PublicKey de l'expéditeur
// unwrapped.rumor → UnsignedEvent avec le contenu en clair
// (Le seal est vérifié automatiquement — signature secp256k1)
```

---

## Scénario de bout en bout

```
# Alice initialise son identité
Terminal 1$ rr init
→ ✅ Identité créée
→ npub1alice...
→ SEED PHRASE (à noter)

# Bob aussi
Terminal 2$ rr init
→ npub1bob...

# Alice ajoute Bob comme contact
Terminal 1$ rr add-contact npub1bob... bob
→ ✅ Contact ajouté : bob (npub1bob...)

# Bob ajoute Alice
Terminal 2$ rr add-contact npub1alice... alice
→ ✅ Contact ajouté : alice (npub1alice...)

# Alice envoie un message
Terminal 1$ rr send bob "Salut, ça marche"
→ 🔐 Envoi à bob (npub1bob...)
→ ✅ Message envoyé (NIP-17 gift wrap kind:1059 sur wss://relay.damus.io)

# Bob synchronise
Terminal 2$ rr sync
→ 🔄 Connexion au relais...
→ 📨 1 nouveau message
→ De: alice — "Salut, ça marche"
→   ✓ Signature vérifiée (secp256k1 via seal)
→   ✓ Déchiffré (NIP-44 ChaCha20-Poly1305)
```

---

## Architecture du code à implémenter

### Flux d'envoi (`cmd_send`)
```
1. Charger identité depuis ~/.rr/keys.json
2. Résoudre le nom de contact → npub (depuis contacts.json)
3. Créer NostrTransport::with_keys(relay_url, user_keys)
4. Appeler message.send(client, receiver_pubkey, content)
5. Afficher confirmation
```

### Flux de réception (`cmd_sync`)
```
1. Charger identité depuis ~/.rr/keys.json
2. Créer NostrTransport::with_keys(relay_url, user_keys)
3. S'abonner aux events Kind::GiftWrap (1059) taggés [p: user_pubkey]
4. Pour chaque event reçu :
   a. client.unwrap_gift_wrap(&event) → UnwrappedGift { sender, rumor }
   b. Extraire rumor.content (le message en clair)
   c. Résoudre sender → nom de contact (depuis contacts.json)
   d. Afficher "De: <nom> — <message>"
```

### Relais
- URL configurable via `~/.rr/config.toml` ou variable d'env `RR_RELAY`
- Défaut : `wss://relay.damus.io`

---

## Fichiers modifiés

| Fichier | Changement |
|---------|------------|
| `crates/rr-core/src/lib.rs` | Ajouter public re-export de `TransportProvider` |
| `crates/rr-core/src/transport/nostr.rs` | Ajouter méthode `relay_url()`, améliorer ergonomie |
| `crates/rr-core/src/message.rs` | ~~STUB~~ → implémentation complète |
| `crates/rr-cli/src/main.rs` | ~~STUB~~ `cmd_send` + `cmd_sync` |
| `Cargo.toml` | Ajouter feature `default-relay` ou config |
| `~/.rr/config.toml` (nouveau) | Configuration relais (optionnel) |

---

## Critère de succès

**Un seul critère** : Alice et Bob, sur deux machines différentes, échangent un message chiffré E2E via NIP-17 (NIP-44 + NIP-59 gift wrap) sur un relais Nostr public, sans aucun serveur intermédiaire à configurer.

**Vérification** : le relais ne peut pas lire le contenu, seule Bob déchiffre, la signature secp256k1 confirme l'expéditeur.

---

## Timeline estimée

| Étape | Durée | Fichiers |
|-------|-------|----------|
| Mise à jour message.rs + transport | 1h | `message.rs`, `transport/nostr.rs` |
| CLI send (avec contact résolution) | 1h30 | `main.rs` |
| CLI sync (subscription + unwrap) | 2h | `main.rs` |
| Config relais (optionnel) | 30min | `config.toml` parsing |
| Test e2e (2 machines) | 2h | Validation manuelle |
| Docs + TRACKING | 30min | `TRACKING.md` |
| **Total** | **~7h30** | **1 jour temps plein** |

---

## Limitations (inchangé)

Mêmes limitations documentées que dans la spec d'origine : pas de forward secrecy (conscient, conditionnel Phase 1+), pas de post-quantum, pas d'anonymat IP. Stack audité par Cure53 (déc. 2023). Voir spec architecture §13.7 pour le plan forward secrecy.
