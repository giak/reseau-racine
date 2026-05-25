# RéseauRacine

Messagerie pair-à-pair chiffrée de bout en bout, basée sur le protocole Nostr (NIP-17).
Racines françaises, réseau mondial.

**Zéro installation sur l'OS** — tout tourne dans Docker.

---

## 1. Prérequis

Un seul logiciel à installer :

- [Docker](https://docs.docker.com/engine/install/) (version 25+)

C'est tout. Pas de Rust, pas de Node, pas de libsodium, pas de GTK.

---

## 2. Premier contact

```bash
git clone https://github.com/reseau-racine/reseau-racine.git
cd reseau-racine
```

À partir d'ici, toutes les commandes Rust s'exécutent dans le container via `./scripts/dev.sh` :

```bash
# Vérifier que tout compile
./scripts/dev.sh cargo check --workspace --exclude rr-tauri

# Lancer les tests
./scripts/dev.sh cargo test --workspace --exclude rr-tauri

# Tout en une commande : check + test + clippy
./scripts/dev.sh sh -c "cargo check --workspace --exclude rr-tauri && cargo test --package rr-core && cargo clippy --package rr-core --package rr-cli"
```

> **Première exécution** : le script détecte que les services Docker ne tournent pas et
> les lance automatiquement. La première compilation télécharge les dépendances Rust
> (2-3 minutes).

---

## 3. Créer son identité

```bash
./scripts/dev.sh cargo run --package rr-cli -- init
```

Le programme va :

1. Générer une paire de clés cryptographique (secp256k1)
2. Sauvegarder la clé privée (afficher un warning si stockage en clair)
3. Afficher votre identité publique (npub)

Exemple de sortie :

```
✅ Identité créée : npub1a2b3c4d5e6f7g8h9i0j...

⚠️  SEULE sauvegarde. Voir la seed phrase ? (oui/non) : oui

SEED PHRASE (notez ces 12 mots sur papier, pas de fichier numérique) :
oiseau montagne clé stylo arbre fenêtre livre nuage soleil rivière lac porte

Stockée dans: /home/user/.local/share/reseau-racine/keys.json

⚠️  Clé stockée en clair sur le disque.
⚠️  Pour plus de sécurité, installe KeePassXC et utilise :
💡  rr init --kdbx ~/vault.kdbx
```

> **La seed phrase est votre seule sauvegarde.** Perdue = identité perdue.
> Notez-la sur papier. Ne la stockez jamais dans un fichier numérique.

Voir son identité :

```bash
./scripts/dev.sh cargo run --package rr-cli -- identity
```

---

## 4. Ajouter un contact

```bash
./scripts/dev.sh cargo run --package rr-cli -- add-contact npub1bob... "Bob"
```

Le contact est stocké dans `~/.rr/contacts.json`.

Lister ses contacts :

```bash
./scripts/dev.sh cargo run --package rr-cli -- contacts
```

---

## 5. Envoyer un message

```bash
./scripts/dev.sh cargo run --package rr-cli -- send bob "Salut !"
```

Le message est :
1. Chiffré avec la clé publique du destinataire (NIP-44)
2. Emballé dans un GiftWrap (kind 1059)
3. Publié sur le relais Nostr

Recevoir les messages en temps réel :

```bash
./scripts/dev.sh cargo run --package rr-cli -- sync
```

Les messages arrivent en direct. `Ctrl+C` pour arrêter.

Architecture :

```
┌─ Vous ─────────────────────┐       ┌─ Relais Nostr ────┐       ┌─ Contact ──────────────┐
│                             │       │                   │       │                        │
│  Message clair              │       │  Kind 1059        │       │  Unwrap + déchiffre    │
│  → Rumor (kind 14)          │      │  (GiftWrap)       │       │  → Message clair       │
│  → Seal (kind 13)           │──────│  Chiffré NIP-44   │──────│                        │
│  → GiftWrap (kind 1059)     │       │  Publique         │       │                        │
│  ← Chiffré E2E →            │       │  Anonyme          │       │                        │
└─────────────────────────────┘       └───────────────────┘       └────────────────────────┘
```

NIP-44 (ChaCha20-Poly1305) garantit le chiffrement E2E.
NIP-17 (GiftWrap) garantit que même le relais ne sait pas qui parle à qui.

---

## 6. Groupes (cellules chiffrées)

Communication chiffrée en petit groupe (3-5 membres) avec clé partagée X25519 + Sender Keys (Signal-style) pour le forward secrecy par message.

### Créer une cellule

```bash
./scripts/dev.sh cargo run --package rr-cli -- group create --label "Famille" --members npub1bob...,npub1alice...
```

### Lister ses cellules

```bash
./scripts/dev.sh cargo run --package rr-cli -- group list
```

### Détails d'une cellule

```bash
./scripts/dev.sh cargo run --package rr-cli -- group info <cell_id>
```

### Inviter un membre

```bash
./scripts/dev.sh cargo run --package rr-cli -- group invite <cell_id> --member npub1charlie...
```

### Envoyer un message

```bash
./scripts/dev.sh cargo run --package rr-cli -- group send <cell_id> --message "Salut tout le monde !"
```

### Écouter les messages

```bash
# Écouter une cellule spécifique
./scripts/dev.sh cargo run --package rr-cli -- group listen <cell_id>

# Mode découverte (auto-crée les cellules inconnues)
./scripts/dev.sh cargo run --package rr-cli -- group listen
```

### Retirer un membre (avec rotation de clés)

```bash
./scripts/dev.sh cargo run --package rr-cli -- group remove <cell_id> --member npub1bob...
```

### Régénérer les clés manuellement

```bash
./scripts/dev.sh cargo run --package rr-cli -- group rotate-key <cell_id>
```

Architecture :

```
┌─ Cellule ───────────────────────────────────────┐
│  Clé partagée X25519 (legacy)                   │
│  + Sender Keys (ratchet HKDF-SHA256 par membre) │
│                                                  │
│  ┌─ Message ─────────────────────────────────┐  │
│  │ ChaCha20-Poly1305 (nonce=0, clé unique)   │  │
│  │ Rumor kind 13, tag h = cell UUID          │  │
│  │ Gift-wrap kind 1059 par destinataire      │  │
│  └────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────┘
```

---

## 7. Restaurer une identité

```bash
./scripts/dev.sh cargo run --package rr-cli -- restore "oiseau montagne clé stylo arbre fenêtre livre nuage soleil rivière lac porte"
```

La seed phrase de 12 mots régénère exactement la même paire de clés.

---

## 8. KeePassXC (coffre-fort de clés)

Par défaut, la clé privée est stockée en clair dans `~/.local/share/reseau-racine/keys.json`.
Pour une sécurité renforcée, `rr` peut stocker les clés dans une base KeePassXC.

### Initialisation directe dans KeePassXC

```bash
# Créer d'abord la base via l'interface KeePassXC
# Puis :
./scripts/dev.sh cargo run --package rr-cli -- init --kdbx ~/vault.kdbx

# rr détecte keepassxc-cli, te demande le mot de passe,
# et sauvegarde automatiquement la config
```

### Migrer une identité existante

```bash
./scripts/dev.sh cargo run --package rr-cli -- export --kdbx ~/vault.kdbx
export RR_KEYSTORE=keepassxc://~/vault.kdbx/Nostr/Identity
```

La variable d'env `RR_KEYSTORE` ou le fichier `~/.config/reseau-racine/config.toml`
définit le backend : `file`, `keepassxc://<db>/<entry>`, ou `keepass-rs://<db>/<entry>`.

---

## 9. Développer

### Architecture du code

```
crates/
├── rr-core/               # Bibliothèque fondamentale
│   ├── crypto.rs          # Chiffrement NIP-44
│   ├── identity.rs        # Clés, seed phrase, nsec/npub
│   ├── message.rs         # NIP-17 (Rumor → Seal → GiftWrap)
│   ├── cell.rs            # Cellules de groupe (Cell, CellMember, SenderKey)
│   ├── cell_transport.rs  # Transport cellules (create, invite, send, listen, remove, rotate)
│   ├── sender_key.rs      # HKDF ratchet + ChaCha20-Poly1305 per-message
│   └── transport/         # Connexion aux relais Nostr
├── rr-cli/                # Interface en ligne de commande
├── rr-stress/             # Simulation de charge (benchmarks)
└── rr-tauri/              # Application de bureau (Tauri v2)
```

### Lancer les tests

```bash
# Tests unitaires
./scripts/dev.sh cargo test --package rr-core

# Vérifications complètes
./scripts/dev.sh cargo clippy --workspace --exclude rr-tauri
./scripts/dev.sh cargo fmt --all --check

# Vérifications de sécurité (Phase 2)
./scripts/dev.sh cargo llvm-cov --workspace --exclude rr-tauri --lcov --output-path coverage/lcov.info
./scripts/dev.sh cargo install cargo-mutants --locked && ./scripts/dev.sh cargo mutants --workspace --exclude rr-tauri
./scripts/dev.sh cargo install auditable2cdx --locked && ./scripts/dev.sh cargo auditable build --package rr-cli --release --locked && ./scripts/dev.sh auditable2cdx target/release/rr > sbom-cyclonedx.json
```

### Services Docker

Le DevContainer monte automatiquement :

| Service | Accès | Description |
|---------|-------|-------------|
| **dev** | `shell` | Rust + GTK + libsodium + outils |
| **nostr-relay** | `ws://nostr-relay:8080` | Relais Nostr local pour tests |

Logs :

```bash
docker compose -f .devcontainer/compose.yaml logs -f nostr-relay
```

---

## 10. Dépannage

| Symptôme | Cause | Solution |
|----------|-------|----------|
| `cargo: command not found` | Docker pas lancé | `./scripts/dev.sh` lance les services auto |
| `Permission denied` | Fichiers ~/.rr verrouillés | `chmod 600 ~/.rr/keys.json` |
| La seed phrase ne restaure pas | Faute de frappe | Vérifiez l'ordre exact des 12 mots |
| `ws://nostr-relay:8080` ne répond pas | Relais pas prêt | `docker compose logs nostr-relay` |

---

## Feuille de route

| EPIC | Status | Description |
|------|--------|-------------|
| 0 | ✅ | Fondations : crypto, identité, CI/CD, sécurité |
| 1 | ✅ | Messagerie P2P : envoi/sync via NIP-17 |
| 2 | ✅ | Groupes & cellules (Sender Keys, rotation) |
| 5 | ✅ | Forward Secrecy (HKDF ratchet + ChaCha20-Poly1305) |
| 7 | ✅ | KeePassXC vault : sécurité des clés |
| 8 | ✅ | Benchmarks performance |
| 9 | ✅ | Simulation charge (rr-stress) |
| 3 | ⬜ | Reticulum WiFi (transport mesh) |
| 4 | ⬜ | Interface graphique (Tauri) |
| 6 | ⬜ | Nœud relais embarqué |
| SEC-1 | ✅ | Sécurité Fixes : nonce ChaCha20, auth rotation, store atomique |

---

## Licence

AGPL-3.0-or-later — voir [LICENSE](LICENSE).
