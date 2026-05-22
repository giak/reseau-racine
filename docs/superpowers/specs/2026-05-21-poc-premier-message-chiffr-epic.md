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
│  1. Écrit "Salut, ça marche"                                       │
│  2. Crée un Rumor (kind 14) — message en clair                     │
│  3. Signe avec clé éphémère → Seal (kind 13)                       │
│  4. Chiffre avec NIP-44 V2 (ChaCha20-Poly1305) → GiftWrap (1059)   │
│  5. Publie sur relais Nostr (WebSocket)                            │
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
│  1. Reçoit GiftWrap (kind 1059)                                    │
│  2. Déchiffre avec NIP-44 V2 (sa clé privée)                       │
│  3. Ouvre le Seal (kind 13) → vérifie signature éphémère            │
│  4. Extrait le Rumor (kind 14)                                     │
│  5. Affiche : "Message de alice: Salut, ça marche"                 │
│     ✓ Signature vérifiée (secp256k1)                               │
│     ✓ Déchiffrement réussi (ChaCha20-Poly1305 AEAD)                │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Scope

### Ce que le POC fait

1. **Génère une identité** — 1 clic → paire de clés secp256k1 (nsec/npub Nostr)
2. **Chiffre un message E2E** — Alice chiffre avec NIP-44 V2 (ChaCha20-Poly1305), enveloppe avec NIP-59 gift wrap, publie sur un relais Nostr
3. **Reçoit et déchiffre** — Bob reçoit le gift wrap via subscription, unwrap, déchiffre NIP-44, vérifie la signature secp256k1
4. **Affiche le résultat** — "Message reçu de Alice : [contenu]"

### Ce que le POC ne fait PAS

- Pas d'UI graphique (CLI suffit)
- Pas de Reticulum, pas de LoRa, pas de Meshtastic
- Pas de PeerTube, pas de vidéos, pas de streams
- Pas de gouvernance, pas de cellules, pas de web of trust
- Pas de cache, pas de P2P, pas de IPFS
- Pas de détection de dégradation
- Pas de groupes, pas de canaux
- **Pas de post-quantum** (limitation secp256k1 — voir § Limitations)

**Un seul transport** : internet via Nostr. Le "multi-transport" viendra après.

---

## Stack du POC

| Composant | Choix | Pourquoi |
|-----------|-------|----------|
| **Langage** | Rust | Pure Rust, binaire unique, sécurité mémoire |
| **Crypto** | Crate `nostr` v0.44.2 (secp256k1 + NIP-44 ChaCha20-Poly1305 + NIP-59) | Support natif NIP-17, pas de FFI C |
| **Transport** | Nostr (NIP-17 pour messages privés, NIP-01 pour relays) | Standard actuel, remplace NIP-04 déprécié |
| **Interface** | CLI (`cargo run`) | Pas de frontend à build, test rapide |
| **Stockage** | Fichier JSON local | Pas de DB, juste `keys.json` et `messages.json` |
| **Async** | tokio + tokio-tungstenite | WebSocket async vers relais Nostr |

### Pourquoi pas NIP-04 (kind 4) ?

NIP-04 est marqué **`unrecommended`** et **déprécié** :
- Utilise AES-256-CBC (pas AEAD, pas d'authentification du ciphertext)
- Utilise secp256k1 ECDH non standard (X coordinate only, pas de hash)
- Fuites de métadonnées (qui envoie à qui est visible dans les tags)
- **CVE-2026-41301** — attaques par messages forgés
- Avertissement officiel : "must not be used for anything you really need to keep secret"

NIP-17 (via NIP-44 + NIP-59) est le standard actuel :
- ChaCha20-Poly1305 (AEAD authentifié)
- X25519 ECDH standard
- Gift wrap (NIP-59) cache les métadonnées
- Support natif dans la crate `nostr` v0.44.2

---

## Scénario de test

```
Terminal 1 (Alice)                    Terminal 2 (Bob)
──────────────                        ──────────────
$ rr init                             $ rr init
→ npub1alice...                       → npub1bob...

$ rr add-contact npub1bob...
→ Contact "bob" ajouté

$ rr send bob "Salut, ça marche"
→ Message envoyé (NIP-17) sur relais
  Gift wrap kind: 1059

                                      $ rr sync
                                      → 1 gift wrap reçu
                                      → Unwrapped: message de alice
                                      → "Salut, ça marche"
                                      ✓ Signature vérifiée (secp256k1)
                                      ✓ Déchiffré (NIP-44 ChaCha20-Poly1305)
```

---

## Format du message POC (NIP-17)

Le message passe par 3 couches :

```
┌─────────────────────────────────────────────────────────────┐
│  COUCHE 1 — RUMOR (kind 14 — PrivateDirectMessage)          │
│  Le message réel, en clair (sera chiffré + signé)           │
│                                                             │
│  {                                                          │
│    "kind": 14,                                              │
│    "content": "Salut, ça marche",                           │
│    "tags": [["p", "npub_bob"]],                             │
│    "created_at": 1716300000                                 │
│  }                                                          │
└──────────────────────────┬──────────────────────────────────┘
                           │ signé avec clé éphémère
┌──────────────────────────▼──────────────────────────────────┐
│  COUCHE 2 — SEAL (kind 13)                                  │
│  Le rumor signé avec une clé secp256k1 ÉPHÉMÈRE             │
│  (pas la clé principale d'Alice)                            │
│                                                             │
│  {                                                          │
│    "kind": 13,                                              │
│    "pubkey": "<clé éphémère_alice>",                        │
│    "content": "<rumor JSON>",                               │
│    "sig": "<signature secp256k1>"                           │
│  }                                                          │
│                                                             │
│  → Le relais voit une signature mais ne peut pas            │
│    la lier à l'identité réelle d'Alice                      │
└──────────────────────────┬──────────────────────────────────┘
                           │ chiffré NIP-44 V2 (ChaCha20-Poly1305)
┌──────────────────────────▼──────────────────────────────────┐
│  COUCHE 3 — GIFT WRAP (kind 1059)                           │
│  Le seal chiffré avec la clé publique de Bob                │
│  Seule métadonnée visible : le destinataire                 │
│                                                             │
│  {                                                          │
│    "kind": 1059,                                            │
│    "pubkey": "<clé éphémère>",                              │
│    "content": "<chiffré NIP-44 V2>",                        │
│    "tags": [["p", "npub_bob"]]  ← SEUL tag visible          │
│    "created_at": 1716300000                                 │
│  }                                                          │
│                                                             │
│  → Le relais voit : "quelqu'un envoie un blob à Bob"        │
│  → Le relais ne voit PAS : qui, quoi, quand exactement      │
└─────────────────────────────────────────────────────────────┘
```

---

## Limitations de sécurité — Honnêteté totale

### Ce que NIP-44 NE PROTÈGE PAS

| Propriété | Statut NIP-44 | Impact pour nous |
|-----------|--------------|------------------|
| **Confidentialité du contenu** | ✅ Oui (ChaCha20-Poly1305 AEAD) | Le contenu est sécurisé |
| **Authentification de l'expéditeur** | ✅ Oui (signature secp256k1 dans le seal) | On peut vérifier qui a envoyé |
| **Intégrité du message** | ✅ Oui (MAC Poly1305) | Le message n'est pas modifiable |
| **Forward secrecy** | ❌ **NON** (Phase 0) | NIP-44 seul ne fournit pas. Prérequis conditionnel en Phase 1+ (voir spec architecture §13.7). |
| **Post-compromise security** | ❌ **NON** (Phase 0) | Idem. |
| **Déni plausible** | ❌ **NON** | On peut prouver qu'Alice a envoyé le message (signature dans le seal) |
| **Protection post-quantique** | ❌ **NON** | Un ordinateur quantique pourrait déchiffrer |
| **Anonymat IP** | ❌ **NON** | Le relais voit l'IP d'Alice et de Bob |
| **Taille du message** | ⚠️ Partiel (padding) | La taille approximative est visible |

### Audit Cure53 (déc. 2023)

L'audit officiel de NIP-44 par Cure53 a trouvé :

| ID | Finding | Sévérité |
|----|---------|----------|
| NOS-01-006 | Lack of forward secrecy | Medium |
| NOS-01-005 | Missing range checks | Medium |
| NOS-01-004 | Timing differences in HMAC comparison | Low |
| NOS-01-007 | Lack of key separation (signing + encryption) | Low |

### Conclusion honnête

**Le POC utilise NIP-44 + NIP-17 + NIP-59, stack audité par Cure53.** Pas de forward secrecy — décision consciente (voir spec architecture §13.7). L'architecture de chiffrement est abstraite derrière un trait Rust, permettant d'ajouter Double Ratchet en Phase 1+ sans changer le transport ni le format événement.

Ce qui reste non couvert :
| Propriété | Statut |
|-----------|--------|
| **Forward secrecy** | ❌ Phase 0. Conditionnel Phase 1+ (§13.7). |
| **Post-compromise** | ❌ Phase 0. Conditionnel Phase 1+. |
| **Post-quantum** | ❌ secp256k1 vulnérable à Shor. NIP-44 v3 en discussion. |
| **Anonymat IP** | ❌ le relais voit l'IP. Reticulum résoudra ça en Phase 3. |
| **Déni plausible** | ❌ Phase 0 (seal signé par clé réelle d'Alice). ✅ Phase 1+ DR (seal signé par clé éphémère, rumor non signé). |
| **Métadonnées** | ⚠️ NIP-59 gift wrap cache qui parle à qui, mais pas le timing.

---

## Critère de succès

**Un seul critère** : Alice et Bob, sur deux machines différentes, échangent un message chiffré E2E via NIP-17 (NIP-44 + NIP-59 gift wrap + ChaCha20-Poly1305) sur un relais Nostr public, sans aucun serveur intermédiaire à configurer.

**Vérification** : le relais ne peut pas lire le contenu (gift wrap + NIP-44), seule Bob peut déchiffrer, la signature secp256k1 confirme l'expéditeur.

Si ça marche → le concept est validé sur un socle stable et audité. La forward secrecy sera ajoutée en Phase 1+ si les conditions sont remplies (§13.7).
Si ça ne marche pas → on corrige avant de construire quoi que ce soit d'autre.

---

## Timeline estimée

| Étape | Durée | Détail |
|-------|-------|--------|
| Setup projet Rust + crate `nostr` | 1h | Cargo init, deps, tokio |
| Génération de clés secp256k1 | 1h | nsec/npub, storage JSON |
| Message NIP-17 (NIP-44 + NIP-59) | 3h | Encrypt, gift wrap, publish |
| Réception message (unwrap + decrypt) | 2h | Subscription, unwrap, NIP-44 decrypt |
| CLI ergonomique | 2h | `init`, `add-contact`, `send`, `sync` |
| Test end-to-end (2 machines) | 2h | Validation |
| Abstraction trait | 2h | `EncryptionProvider` trait + `Nip44Provider` impl (prépare migration DR future) |
| **Total** | **~13h** | **2 jours à temps plein** |

---

## Prochaines étapes après le POC

Une fois le POC validé, on ajoute dans l'ordre :

1. **Groupes & Cellules** (NIP-44 + clé de groupe X25519, cellules de 3)
2. **Double Ratchet** (conditionnel : audit ou merge rust-nostr) — forward secrecy + post-compromise + sender-keys
3. **Auto-destruction** (TTL via NIP-40)
4. **Client Tauri** (UI graphique)
5. **Reticulum WiFi** (second transport, même message chiffré, via leviculum Rust)
6. **Détection de dégradation** (bascule auto internet → Reticulum)
7. **Nœud relais** (Pi 5 + cache + IPFS)
8. **Publication** (PeerTube + Owncast)

Chaque étape est un EPIC séparé. On ne construit pas tout en même temps. Forward secrecy n'est pas dans le POC — décision consciente. L'architecture permet de l'ajouter plus tard sans changer le transport ni l'identité.
