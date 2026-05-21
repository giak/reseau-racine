# EPIC — POC Fil Rouge : "Premier Message Chiffré"

> **Objectif** : prouver que l'identité cryptographique + chiffrement E2E + transport interchangeable fonctionne. Rien d'autre.

---

## Scope

### Ce que le POC fait

1. **Génère une identité** — 1 clic → paire de clés secp256k1 (nsec/npub Nostr)
2. **Envoie un message E2E** — Alice publie un message privé via NIP-17 (NIP-44 + NIP-59 gift wrap) sur un relais Nostr
3. **Reçoit et déchiffre** — Bob reçoit le gift wrap, l'ouvre, vérifie la signature, lit le message
4. **Affiche le résultat** — "Message reçu de Alice : [contenu]"

### Ce que le POC ne fait PAS

- Pas d'UI graphique (CLI suffit)
- Pas de Reticulum, pas de LoRa, pas de Meshtastic
- Pas de PeerTube, pas de vidéos, pas de streams
- Pas de gouvernance, pas de cellules, pas de web of trust
- Pas de cache, pas de P2P, pas de IPFS
- Pas de détection de dégradation
- Pas de groupes, pas de canaux

**Un seul transport** : internet via Nostr. Le "multi-transport" viendra après.

---

## Stack du POC

| Composant | Choix | Pourquoi |
|-----------|-------|----------|
| **Langage** | Rust | Pure Rust, binaire unique, sécurité mémoire |
| **Crypto** | Crate `nostr` v0.44.2 (secp256k1 + NIP-44 ChaCha20-Poly1305 + NIP-59 gift wrap) | Support natif NIP-17, pas de FFI C |
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
                                      ✓ Déchiffré (ChaCha20-Poly1305)
```

---

## Format du message POC (NIP-17)

Le message passe par 3 couches :

```
1. Message (kind 14 — PrivateDirectMessage)
   {
     "kind": 14,
     "content": "Salut, ça marche",
     "tags": [["p", "npub_bob"]]
   }

2. Seal (kind 13 — signé avec une clé éphémère)
   {
     "kind": 13,
     "content": "<rumor du kind 14>",
     "pubkey": "<clé éphémère d'Alice>"
   }

3. Gift Wrap (kind 1059 — chiffré NIP-44 pour Bob)
   {
     "kind": 1059,
     "pubkey": "<clé éphémère>",
     "content": "<chiffré NIP-44: ChaCha20-Poly1305>",
     "tags": [["p", "npub_bob"]]  ← seule métadonnée visible
   }
```

Le relais ne voit que le gift wrap (kind 1059). Il ne sait pas qui est l'expéditeur réel, ni le contenu. Bob seul peut unwrap.

---

## Critère de succès

**Un seul critère** : Alice et Bob, sur deux machines différentes, échangent un message chiffré E2E via NIP-17 (gift wrap + ChaCha20-Poly1305) sur un relais Nostr public, sans aucun serveur intermédiaire à configurer.

Si ça marche → le concept est validé. On itère.
Si ça ne marche pas → on corrige avant de construire quoi que ce soit d'autre.

---

## Timeline estimée

| Étape | Durée | Détail |
|-------|-------|--------|
| Setup projet Rust + crate `nostr` | 1h | Cargo init, deps, tokio |
| Génération de clés secp256k1 | 1h | nsec/npub, storage JSON |
| Envoi message (NIP-17 publish) | 3h | Crate `nostr` send_private_msg + WebSocket |
| Réception message (unwrap gift wrap) | 3h | WebSocket subscription + unwrap_gift_wrap |
| CLI ergonomique | 2h | `init`, `add-contact`, `send`, `sync` |
| Test end-to-end (2 machines) | 2h | Validation |
| **Total** | **~12h** | **1-2 jours à temps plein** |

---

## Prochaines étapes après le POC

Une fois le POC validé, on ajoute dans l'ordre :

1. **Groupes** (clé de groupe X25519, cellules de 3)
2. **Forward secrecy** (ratchet)
3. **Auto-destruction** (TTL via NIP-40)
4. **Reticulum WiFi** (second transport, même message chiffré)
5. **Détection de dégradation** (bascule auto internet → Reticulum)
6. **Client Tauri** (UI graphique)
7. **Nœud relais** (Pi 5 + cache + IPFS)

Chaque étape est un EPIC séparé. On ne construit pas tout en même temps.
