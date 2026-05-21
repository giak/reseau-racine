# EPIC — POC Fil Rouge : "Premier Message Chiffré"

> **Objectif** : prouver que l'identité cryptographique + chiffrement E2E + transport interchangeable fonctionne. Rien d'autre.

---

## Scope

### Ce que le POC fait

1. **Génère une identité** — 1 clic → paire de clés (nsec/npub Nostr)
2. **Envoie un message E2E** — Alice chiffre avec la clé publique de Bob → signe avec sa clé privée → publie sur un relais Nostr
3. **Reçoit et déchiffre** — Bob voit le message sur le relais → vérifie la signature → déchiffre
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
| **Langage** | Rust | libsodium bindings, performance, binaire unique |
| **Crypto** | libsodium (X25519 + Ed25519 + AES-256-GCM) | Standard, audité |
| **Transport** | Nostr (nip-04 pour chiffrement, nip-01 pour relays) | Écosystème existant, relais publics gratuits |
| **Interface** | CLI (`cargo run`) | Pas de frontend à build, test rapide |
| **Stockage** | Fichier JSON local | Pas de DB, juste `keys.json` et `messages.json` |

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
→ Message envoyé sur relay.nostr1.com
  ID: abc123

                                      $ rr sync
                                      → 1 nouveau message de alice
                                      → "Salut, ça marche"
                                      ✓ Signature vérifiée (alice)
                                      ✓ Déchiffré avec succès
```

---

## Format du message POC

```json
{
  "kind": 4,
  "pubkey": "npub_alice",
  "created_at": 1716300000,
  "content": "<chiffré nip-04: base64 + ?iv=...>",
  "tags": [["p", "npub_bob"]],
  "sig": "<signature Ed25519>"
}
```

Format Nostr kind 4 (DM chiffré). Déjà supporté par les relais existants. Pas besoin de serveur custom.

---

## Critère de succès

**Un seul critère** : Alice et Bob, sur deux machines différentes, échangent un message chiffré E2E via un relais Nostr public, sans aucun serveur intermédiaire à configurer.

Si ça marche → le concept est validé. On itère.
Si ça ne marche pas → on corrige avant de construire quoi que ce soit d'autre.

---

## Timeline estimée

| Étape | Durée | Détail |
|-------|-------|--------|
| Setup projet Rust + libsodium | 2h | Cargo init, deps, CI |
| Génération de clés | 2h | nsec/npub, storage JSON |
| Envoi message (chiffrement + publish) | 4h | X25519 + Ed25519 + WebSocket vers relais |
| Réception message (sync + verify + decrypt) | 4h | WebSocket subscription + verify + decrypt |
| CLI ergonomique | 2h | `init`, `add-contact`, `send`, `sync` |
| Test end-to-end (2 machines) | 2h | Validation |
| **Total** | **~16h** | **2-3 jours à temps plein** |

---

## Prochaines étapes après le POC

Une fois le POC validé, on ajoute dans l'ordre :

1. **Groupes** (clé de groupe X25519, cellules de 3)
2. **Forward secrecy** (ratchet)
3. **Auto-destruction** (TTL)
4. **Reticulum WiFi** (second transport, même message chiffré)
5. **Détection de dégradation** (bascule auto internet → Reticulum)
6. **Client Tauri** (UI graphique)
7. **Nœud relais** (Pi 5 + cache + IPFS)

Chaque étape est un EPIC séparé. On ne construit pas tout en même temps.
