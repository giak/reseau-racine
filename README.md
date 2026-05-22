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
2. Sauvegarder la clé privée dans `~/.rr/keys.json` (permissions 0600)
3. Afficher votre identité publique (npub)

Exemple de sortie :

```
✅ Identité créée
npub: npub1a2b3c4d5e6f7g8h9i0j...

⚠️  La seed phrase suivante est votre SEULE sauvegarde. Prêt à voir ? (oui/non) : oui

SEED PHRASE (notez ces 12 mots sur papier, pas de fichier numérique) :
oiseau montagne clé stylo arbre fenêtre livre nuage soleil rivière lac porte

Stockée dans: /home/user/.rr/keys.json
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

## 5. Envoyer un message (bientôt)

Les commandes `send` et `sync` sont en cours de développement (EPIC 1).

En attendant, l'architecture est prête :

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

## 6. Restaurer une identité

```bash
./scripts/dev.sh cargo run --package rr-cli -- restore "oiseau montagne clé stylo arbre fenêtre livre nuage soleil rivière lac porte"
```

La seed phrase de 12 mots régénère exactement la même paire de clés.

---

## 7. Développer

### Architecture du code

```
crates/
├── rr-core/          # Bibliothèque fondamentale
│   ├── crypto.rs     # Chiffrement NIP-44
│   ├── identity.rs   # Clés, seed phrase, nsec/npub
│   ├── message.rs    # NIP-17 (Rumor → Seal → GiftWrap)
│   └── transport/    # Connexion aux relais Nostr
├── rr-cli/           # Interface en ligne de commande
└── rr-tauri/         # Application de bureau (Tauri v2)
```

### Lancer les tests

```bash
# Tests unitaires
./scripts/dev.sh cargo test --package rr-core

# Vérifications complètes
./scripts/dev.sh cargo clippy --workspace --exclude rr-tauri
./scripts/dev.sh cargo fmt --all --check
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

## 8. Dépannage

| Symptôme | Cause | Solution |
|----------|-------|----------|
| `cargo: command not found` | Docker pas lancé | `./scripts/dev.sh` lance les services auto |
| `Permission denied` | Fichiers ~/.rr verrouillés | `chmod 600 ~/.rr/keys.json` |
| La seed phrase ne restaure pas | Faute de frappe | Vérifiez l'ordre exact des 12 mots |
| `ws://nostr-relay:8080` ne répond pas | Relais pas prêt | `docker compose logs nostr-relay` |

---

## Feuille de route

- **Phase 0** — Fondations : crypto, identité, transport Nostr ✓
- **EPIC 1** — Messagerie P2P : envoi/sync via NIP-17
- **EPIC 2** — Contacts et discovery
- **EPIC 3** — Groupes et salons
- **EPIC 4** — Interface graphique (Tauri)
- **EPIC 5** — Version mobile

---

## Licence

AGPL-3.0-or-later — voir [LICENSE](LICENSE).
