# EPIC 0 — Infrastructure & Setup : "Fondations"

> **Objectif** : mettre en place tout ce qu'il faut pour que n'importe qui puisse cloner le repo, développer, tester, et build — sans rien installer de complexe sur sa machine.

---

## Principe

- **Docker pour les services** (Nostr relay, IPFS) → infrastructure
- **DevContainer pour les devs** → environnement reproductible (optionnel, VS Code)
- **Binaire natif pour les utilisateurs** → Tauri (.exe/.dmg/.deb), PAS Docker
- **Cargo workspace pour le code** → monorepo Rust, un `Cargo.lock`, deps partagées

---

## Structure du repository

```
reseau-racine/
├── Cargo.toml                    # Workspace root (resolver = "2")
├── deny.toml                     # cargo-deny: advisories, licences, bans
├── rust-toolchain.toml           # Rust version pinned (ex: 1.85.0)
├── .github/
│   └── workflows/
│       ├── ci.yml                # Build + test + clippy + fmt sur push/PR
│       ├── release.yml           # Build Tauri + publish GitHub Releases
│       └── docs.yml              # Build docs rs
├── .devcontainer/
│   ├── devcontainer.json         # VS Code Dev Container config
│   ├── Dockerfile                # Image de dev (Rust + libsodium + outils)
│   └── compose.yaml              # Services: nostr-relay, IPFS, etc.
├── docker/
│   ├── nostr-relay/
│   │   └── Dockerfile            # nostr-rs-relay custom config
│   ├── ipfs/
│   │   └── compose.yaml        # IPFS pinning service
│   └── README.md                 # Comment lancer les services
├── crates/
│   ├── rr-core/                  # Bibliothèque core (crypto, identité, messages)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── crypto.rs         # NIP-44, secp256k1
│   │       ├── identity.rs       # Génération/stockage de clés
│   │       ├── message.rs        # NIP-17: rumor → seal → gift wrap
│   │       └── transport/        # Trait abstrait multi-transport (implémentations futures)
│   │           ├── mod.rs        # Trait TransportProvider
│   │           └── nostr.rs      # Transport internet (WebSocket) — Phase 0
│   ├── rr-cli/                   # CLI pour le POC (binaire)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs           # init, add-contact, send, sync
│   └── rr-tauri/                 # App Tauri (binaire desktop)
│       ├── Cargo.toml
│       ├── src/
│       │   ├── main.rs           # Tauri commands
│       │   └── lib.rs            # Bridge core → UI
│       ├── tauri.conf.json       # Config Tauri v2
│       ├── build.rs
│       ├── icons/                # Icônes de l'app
│       └── capabilities/
│           └── default.json      # Permissions frontend (OBLIGATOIRE Tauri v2)
├── ui/                           # Frontend Tauri (React + TypeScript)
│   ├── package.json
│   ├── tsconfig.json
│   ├── vite.config.ts
│   └── src/
│       ├── main.tsx
│       ├── App.tsx
│       └── components/
├── docs/
│   └── superpowers/specs/        # Specs et EPICs
├── tests/
│   ├── e2e/                      # Tests end-to-end (2 machines simulées)
│   ├── integration/              # Tests d'intégration (relais local)
│   ├── nip44_vectors.json        # Vecteurs KAT NIP-44 (vérifiés SHA256)
│   ├── nip44_kat.rs              # Tests KAT: nostr::nip44 conforme au spec
│   └── proptest.rs               # Property-based tests (invariants crypto)
├── scripts/
│   ├── setup.sh                  # Setup dev (installe deps système)
│   ├── test-e2e.sh               # Lance les tests E2E
│   └── release.sh                # Script de release (bump version + tag)
├── AGENTS.md                     # Instructions pour les agents IA
├── CONTRIBUTING.md               # Guide de contribution
└── README.md                     # Documentation principale
```

### `Cargo.toml` (Workspace root)

```toml
[workspace]
resolver = "2"
members = ["crates/*"]

[workspace.package]
edition = "2021"
license = "AGPL-3.0-or-later"
authors = ["RéseauRacine contributors"]

[workspace.dependencies]
nostr = { version = "0.44", features = ["nip44", "nip59", "nip06"] }
nostr-sdk = { version = "0.44", features = ["nip44", "nip59"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
zeroize = { version = "1", features = ["zeroize_derive"] }
tokio-tungstenite = "0.24"
proptest = "1"              # Dev-dependency: property-based tests
rusqlite = "0.39"           # Phase 1+ storage (feature "bundled" ou "bundled-sqlcipher")
```

> `[workspace.dependencies]` (Rust 1.64+) centralise les versions pour toutes les crates du workspace.

### `rust-toolchain.toml`

```toml
[toolchain]
channel = "1.85.0"
components = ["rustfmt", "clippy"]
targets = ["x86_64-unknown-linux-gnu", "aarch64-apple-darwin", "x86_64-apple-darwin", "x86_64-pc-windows-msvc"]
```

> Le fichier est implicite — pas besoin de `cargo install` manuel. Rustup l'utilise automatiquement.

### `dependabot.yml` (`.github/dependabot.yml`)

```yaml
version: 2
updates:
  - package-ecosystem: cargo
    directory: "/"
    schedule:
      interval: weekly
    open-pull-requests-limit: 5
  - package-ecosystem: npm
    directory: "/ui"
    schedule:
      interval: weekly
    open-pull-requests-limit: 5
  - package-ecosystem: github-actions
    directory: "/"
    schedule:
      interval: monthly
```

### Configuration management

Les paramètres utilisateur (URL relais, timeout, API IPFS) sont stockés dans `~/.rr/config.toml` :

```toml
[relay]
url = "wss://relay.reseau-racine.fr"

[ipfs]
api_url = "http://localhost:5001"

[keys]
path = "~/.rr/keys"
```

### Stockage local

Phase 0 : fichiers JSON simples dans `~/.rr/`. Phase 1+ : SQLCipher (rusqlite).

```
~/.rr/
├── config.toml        # Paramètres (URL relais, timeout)
├── keys.json           # Clé privée (seed BIP-39 + nsec), permissions 0600
├── contacts.json       # Carnet d'adresses (npub → nom, relais)
└── data/               # Messages reçus (fichiers JSON par conversation)
    └── <pubkey>.json
```

> Phase 0 : pas de SQLCipher (6 dépendances lourdes, overkill pour POC). JSON + `zeroize` + permissions 0600 suffisent. Les messages sont stockés chiffrés (NIP-44 output). Phase 1+ : migration vers `rusqlite` avec feature `bundled-sqlcipher` (compile SQLCipher from source, pas besoin d'openssl système).

### Backup & recovery (BIP-39 seed phrase)

```rust
use nostr::nips::nip06;

// Phase 0: `rr init` génère une seed phrase de 12 mots
let mnemonic = nip06::Mnemonic::generate(12)?;
let keys = Keys::from_mnemonic(&mnemonic, "")?;

// Affiche seed phrase + nsec
println!("SEED PHRASE (notez ces 12 mots): {}", mnemonic);
println!("nsec (alternative): {}", keys.secret_key().to_bech32()?);

// Restore: rr restore <12words> ou rr restore nsec1...
let restored = Keys::from_mnemonic(&input_mnemonic, "")?;
```

> L'utilisateur note sa seed phrase (12 mots BIP-39) sur papier, pas de fichier numérique. Import : `rr restore "word1 word2 ... word12"`. Phase 1+ : backup chiffré vers relais privé (NIP-44 sur kind 10002).

---

## Ce que chaque composant fait

### `rr-core` (bibliothèque)

| Module | Rôle | Dépendances |
|--------|------|------------|
| `crypto` | NIP-44 V2, ECDH secp256k1, keystore | `nostr` (NIP-44), `zeroize` |
| `identity` | Génération/stockage de clés secp256k1, nsec/npub | `nostr` (Keys, SecretKey), `zeroize` |
| `message` | NIP-17: send_private_msg, unwrap_gift_wrap | `nostr-sdk` (Client), `nostr` (types) |
| `transport::nostr` | WebSocket vers relais Nostr, publish, subscribe | `nostr-sdk` (Client, RelayPool) |

### `rr-cli` (binaire CLI — POC)

| Commande | Rôle |
|----------|------|
| `rr init` | Génère une identité secp256k1, stocke dans `~/.rr/` |
| `rr add-contact <npub> <nom>` | Ajoute un contact au carnet local |
| `rr send <nom> "message"` | Envoie un message via NIP-44 + NIP-17 (gift wrap) |
| `rr sync` | Reçoit les nouveaux messages, unwrap, déchiffre, affiche |
| `rr contacts` | Liste les contacts |
| `rr identity` | Affiche l'identité courante (npub) |

### `rr-tauri` (binaire desktop — app finale)

| Composant | Rôle |
|-----------|------|
| `lib.rs` | Entry point principal — Tauri commands, state management, `#[cfg_attr(mobile, tauri::mobile_entry_point)]` |
| `main.rs` | Minimal — appelle juste `app_lib::run()` (standard Tauri v2) |
| `commands.rs` | Commandes Tauri exposées au frontend (`#[tauri::command]`) |
| `state.rs` | State management (`Mutex`/`RwLock` pour thread safety) |
| `tauri.conf.json` | Config Tauri v2 (identifier, bundle, updater, frontend dev URL) |
| `capabilities/default.json` | Permissions — quelles commandes le frontend peut appeler (OBLIGATOIRE Tauri v2) |
| `build.rs` | `tauri_build::build()` — requis pour le build system Tauri |
| `icons/` | Icônes générées par `tauri icon` |

### `capabilities/default.json` (Exemple Tauri v2)

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "main-capability",
  "description": "Permissions pour la fenêtre principale",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "core:path:default",
    "core:event:default",
    "core:window:default",
    "core:app:default",
    "core:resources:default",
    "core:menu:default",
    "core:tray:default"
  ]
}
```

> Les permissions sont formatées `${plugin}:${command}:${action}`. Les sets `:default` regroupent les permissions raisonnables. Fichier auto-généré dans `gen/schemas/` par `tauri-build`. Phase 1+ (Tauri app) ajoute `shell:allow-open`, `dialog:default`, `fs:allow-read/write`.

### `ui/` (frontend React)

| Composant | Rôle |
|-----------|------|
| `App.tsx` | Router, layout principal |
| `components/` | UI: chat, contacts, identité, paramètres |
| `@tauri-apps/api` | Communication avec le backend Rust |

---

## DevContainer — Environnement de dev reproductible

### Pour qui

Les développeurs. Pas les utilisateurs finaux.

### `devcontainer.json`

```json
{
  "name": "RéseauRacine Dev",
  "dockerComposeFile": ["compose.yaml"],
  "service": "dev",
  "workspaceFolder": "/workspace",
  "customizations": {
    "vscode": {
      "extensions": [
        "rust-lang.rust-analyzer",
        "tamasfe.even-better-toml",
        "vadimcn.vscode-lldb",
        "bradlc.vscode-tailwindcss"
      ]
    }
  },
  "features": {
    "ghcr.io/devcontainers/features/node:1": {
      "version": "lts"
    }
  },
  "postCreateCommand": "cargo build --workspace && cd ui && npm ci"
}
```

### `compose.yaml` (DevContainer)

```yaml
services:
  dev:
    build:
      context: .
      dockerfile: Dockerfile
    volumes:
      - ..:/workspace:cached
    command: sleep infinity
    depends_on:
      nostr-relay:
        condition: service_healthy
      ipfs:
        condition: service_started

  nostr-relay:
    image: scsibug/nostr-rs-relay:latest
    ports:
      - "8080:8080"
    volumes:
      - ./docker/nostr-relay/config.toml:/app/config.toml:ro
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080"]
      interval: 10s
      timeout: 5s
      retries: 3

  ipfs:
    image: ipfs/kubo:latest
    ports:
      - "4001:4001"
      - "5001:5001"
      - "8081:8080"
    volumes:
      - rr-ipfs-data:/data/ipfs
    healthcheck:
      test: ["CMD", "ipfs", "id"]
      interval: 30s
      timeout: 10s
      retries: 3

volumes:
  rr-ipfs-data:
```

### `Dockerfile` (DevContainer)

```dockerfile
# Debian bookworm-slim — 70 Mo vs 120 Mo pour Ubuntu
FROM mcr.microsoft.com/devcontainers/rust:1-bookworm-slim

# Dépendances système pour Tauri v2 + libsodium
# bookworm-slim = Debian minimal, pas de paquets inutiles
RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl-dev \
    libgtk-3-dev \
    libwebkit2gtk-4.1-dev \
    libappindicator3-dev \
    librsvg2-dev \
    patchelf \
    libsodium-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace
```

> **Note** : image dev = `bookworm-slim` (70 Mo). Alternative Alpine impossible — musl libc incompatible avec webkit2gtk. `--no-install-recommends` évite les paquets optionnels. Le vrai poids (~800 Mo) vient des dépendances Tauri, pas de l'image de base. `cargo install tauri-cli` prend ~5min unique à la création de l'image ; alternative `npm install -D @tauri-apps/cli` dans le frontend si le temps de build pose problème.

---

## CI/CD — GitHub Actions

### `ci.yml` — Build + Test sur chaque push/PR

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always

jobs:
  lint:
    name: Lint & Format
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - uses: Swatinem/rust-cache@v2
      - run: cargo fmt --all -- --check
      - run: cargo clippy --all-targets -- -D warnings

  test:
    name: Test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo test --workspace --locked --verbose

  audit:
    name: Security Audit
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo install cargo-deny --locked
      - run: cargo-deny check advisories bans licenses sources

  check-cross:
    name: Check (macOS + Windows)
    strategy:
      matrix:
        os: [macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo check --workspace --locked

  build-cli:
    name: Build CLI (Linux)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo build --package rr-cli --release --locked
      - uses: actions/upload-artifact@v4
        with:
          name: rr-cli-linux
          path: target/release/rr

  build-tauri:
    name: Build Tauri (Linux)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - name: Install system dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y \
            libwebkit2gtk-4.1-dev \
            libappindicator3-dev \
            librsvg2-dev \
            patchelf
      - uses: actions/setup-node@v4
        with:
          node-version: lts/*
          cache: npm
          cache-dependency-path: ui/package-lock.json
      - run: cd ui && npm ci
      - uses: tauri-apps/tauri-action@v0
        with:
          projectPath: crates/rr-tauri
      - uses: actions/upload-artifact@v4
        with:
          name: rr-tauri-linux
          path: target/release/bundle/deb/*.deb
```

### `release.yml` — Release sur tag

```yaml
name: Release

on:
  push:
    tags:
      - "v*"

jobs:
  publish-tauri:
    permissions:
      contents: write
    strategy:
      fail-fast: false
      matrix:
        include:
          - platform: ubuntu-24.04
            args: ""
          - platform: macos-latest
            args: "--target aarch64-apple-darwin"
          - platform: macos-latest
            args: "--target x86_64-apple-darwin"
          - platform: windows-latest
            args: ""
    runs-on: ${{ matrix.platform }}
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node
        uses: actions/setup-node@v4
        with:
          node-version: lts/*
          cache: npm
          cache-dependency-path: ui/package-lock.json

      - name: Install Rust stable
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.platform == 'macos-latest' && 'aarch64-apple-darwin,x86_64-apple-darwin' || '' }}

      - name: Rust cache
        uses: Swatinem/rust-cache@v2
        with:
          workspaces: "./ -> target"

      - name: Install system dependencies (Linux)
        if: matrix.platform == 'ubuntu-24.04'
        run: |
          sudo apt-get update
          sudo apt-get install -y \
            libwebkit2gtk-4.1-dev \
            libappindicator3-dev \
            librsvg2-dev \
            patchelf

      - name: Install frontend deps
        run: cd ui && npm ci

      - name: Build Tauri
        uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
          TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY_PASSWORD }}
        with:
          tagName: ${{ github.ref_name }}
          releaseName: "RéseauRacine ${{ github.ref_name }}"
          releaseBody: "Voir les changements dans le changelog."
          releaseDraft: true
          prerelease: false
          projectPath: crates/rr-tauri
          args: ${{ matrix.args }}
```

---

## Stratégie relais Nostr (NIP-65)

### Phase 0 (POC)

- **1 relais local** (DevContainer `nostr-rs-relay` sur `localhost:8080`)
- **1 relais public configurable** (default `wss://relay.reseau-racine.fr` si déployé, sinon `wss://nos.lol`)

Communication via `nostr-sdk` — subscriptions persistantes (WebSocket), **pas de polling HTTP**. Le client écoute en continu les événements `kind=1059` (gift wrap) via un filtre avec `since=now`. Pas de requête périodique = pas de corrélation temporelle exploitable.

### Phase 1+ (production)

Chaque utilisateur publie un événement `kind:10002` (NIP-65) listant ses relais :

```json
{
  "kind": 10002,
  "tags": [
    ["r", "wss://relay1.reseau-racine.fr", "write"],
    ["r", "wss://relay2.example.com", "read"]
  ]
}
```

- L'expéditeur lit le `kind:10002` du destinataire
- Envoie le gift wrap aux **read relays** du destinataire
- Taille recommandée : 2-4 relais par catégorie

---

## Tauri v2 auto-updater

### Génération des clés (one-time, env de dev)

```bash
npm run tauri signer generate -- -w ~/.tauri/reseau-racine.key
```

Fichiers créés :
- `~/.tauri/reseau-racine.key` → **PRIVÉ**, jamais commité, stocké dans GitHub Secrets
- `~/.tauri/reseau-racine.key.pub` → copié dans `tauri.conf.json` (sûr à commiter)

### Configuration `tauri.conf.json`

```json
{
  "bundle": {
    "createUpdaterArtifacts": true
  },
  "plugins": {
    "updater": {
      "active": true,
      "pubkey": "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IDM4NDVDMEIyQ0ExN0IwRjk...",
      "endpoints": [
        "https://github.com/reseau-racine/reseau-racine/releases/latest/download/latest.json"
      ]
    }
  }
}
```

### CI (GitHub Actions)

Les secrets nécessaires :
- `TAURI_SIGNING_PRIVATE_KEY` → contenu de `reseau-racine.key`
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` → mot de passe (optionnel)

Le workflow `release.yml` (déjà défini) utilise ces secrets via `tauri-action@v0`. L'action génère automatiquement le `latest.json` avec les signatures et l'upload sur GitHub Releases.

### Vérification

- Signature Ed25519, vérifiée avant installation
- HTTPS uniquement (TLS enforced)
- Échec de vérification = mise à jour annulée
- `latest.json` contient : version, notes, pub_date, signatures + URLs par plateforme

---

## Docker — Services d'infrastructure

### Ce qui tourne dans Docker

| Service | Image | Usage |
|---------|-------|-------|
| **nostr-rs-relay** | `scsibug/nostr-rs-relay` | Relais Nostr local pour tests et production |
| **IPFS (kubo)** | `ipfs/kubo` | Pinning de contenu, mirror d'articles |

> PeerTube (hébergement vidéo) et Owncast (streaming) sont déployés en Phase 2 — pas dans l'infra de dev initiale.

### Ce qui NE tourne PAS dans Docker

| Composant | Packaging | Pourquoi |
|-----------|-----------|----------|
| **rr-cli** | Binaire Rust (`cargo install`) | CLI native, pas besoin de container |
| **rr-tauri** | `.exe` / `.dmg` / `.deb` | App desktop native, double-clic |
| **rr-core** | Bibliothèque Rust (`cargo add`) | Dépendance dans d'autres crates |

---

## Critères de succès de l'EPIC 0

| Critère | Vérification |
|---------|-------------|
| **`git clone` + `cargo build`** fonctionne | Sur Linux, macOS, Windows (WSL2) |
| **`cargo test --workspace`** passe | 100% des tests passent |
| **Tests KAT NIP-44** inclus | Vecteurs de test officiels du NIP-44 V2 |
| **`cargo fmt --check`** passe | Code formaté |
| **`cargo clippy`** passe | Pas de warnings |
| **`cargo-deny`** passe | Aucun advisory, license, ou ban violé |
| **DevContainer** se lance | `docker compose up -d` dans `.devcontainer/` |
| **CI green** sur push | GitHub Actions: lint + test + audit + build |
| **POC CLI** fonctionne | `rr init` → `rr send` → `rr sync` entre 2 terminaux |
| **README** documente le setup | Un novice peut suivre les étapes |

---

## Stratégie de test

| Type | Outil | Couvre |
|------|-------|--------|
| **Unitaires** | `cargo test` | Crypto (NIP-44 encrypt/decrypt, ECDH), messages (wrap/unwrap), identity (clés, nsec) |
| **KAT (Known Answer Tests)** | `nip44_vectors.json` + `nip44_kat.rs` | Vecteurs officiels NIP-44 V2 (paulmillr/nip44, SHA256:269ed0f6). Vérifie que nostr::nip44 encrypt/decrypt produit les outputs conformes. |
| **Property-based** | `proptest` (ajouté à `rr-core`) | Invariants : `decrypt(encrypt(m)) == m`, clé différente → échec, messages extrêmes (1 octet à 64KB), messages vides rejetés |
| **Intégration** | `tests/integration/` | Client serveur nostr local, round-trip message |
| **E2E** | `tests/e2e/` | 2 processus CLI simulés (Alice → relais → Bob), validation du flux complet |

> Les tests KAT sont **obligatoires** pour la crypto. `nip44_vectors.json` est téléchargé depuis [paulmillr/nip44](https://github.com/paulmillr/nip44/blob/master/nip44.vectors.json) et vérifié contre SHA256 `269ed0f69e4c192512cc779e78c555090cebc7c785b609e338a62afc3ce25040`. Le fichier contient les vecteurs `valid.encrypt_decrypt`, `invalid.decrypt`, `valid.get_conversation_key`, etc.
>
> **Property-based** : `proptest` génère des messages aléatoires (1 octet à 64KB, UTF-8, binaire). Invariants testés : `decrypt(encrypt(m, k_alice, k_bob), k_bob, k_alice) == m`, clé différente → erreur, padding valide après round-trip. Lancement : `PROPTEST_CASES=1000 cargo test --test proptest`.

## Timeline estimée

| Étape | Durée | Détail |
|-------|-------|--------|
| Structure workspace + crates | 2h | Cargo.toml, rr-core, rr-cli, rr-tauri |
| DevContainer + Docker Compose | 3h | Dockerfile, compose.yaml, devcontainer.json |
| CI/CD GitHub Actions | 3h | ci.yml, release.yml, rust-cache |
| rr-core: crypto + identity | 4h | secp256k1, NIP-44, génération de clés |
| rr-core: message NIP-17 | 4h | rumor → seal → gift wrap, unwrap |
| rr-core: transport Nostr | 3h | WebSocket, publish, subscribe |
| rr-cli: CLI complet | 3h | init, add-contact, send, sync |
| Tests + docs | 3h | Tests unitaires, integration, README |
| **Total** | **~25h** | **3-4 jours à temps plein** |

---

## Prochaines étapes après l'EPIC 0

1. **EPIC 1 — POC "Premier Message Chiffré"** (14h) — Alice et Bob échangent un message via NIP-17 + NIP-44 + NIP-59
2. **EPIC 2 — Groupes & Cellules** — NIP-44 + clé de groupe X25519, cellules de 3 (Double Ratchet optionnel via sender-keys si conditions remplies)
3. **EPIC 3 — Reticulum WiFi** — Second transport, bascule automatique
4. **EPIC 4 — Client Tauri** — UI React, chat, contacts, notifications
5. **EPIC 5 — Forward Secrecy** — Double Ratchet (si conditions remplies : audit OU merge rust-nostr OU NIP standardisé)
6. **EPIC 6 — Nœud Relais** — Pi 5 + Docker Compose + cache + IPFS
