# EPIC 0 — Infrastructure & Setup : "Fondations"

> **Objectif** : mettre en place tout ce qu'il faut pour que n'importe qui puisse cloner le repo, développer, tester, et build — sans rien installer de complexe sur sa machine.

---

## Principe

- **Docker pour les services** (Nostr relay, PeerTube, IPFS) → infrastructure
- **DevContainer pour les devs** → environnement reproductible (optionnel, VS Code)
- **Binaire natif pour les utilisateurs** → Tauri (.exe/.dmg/.deb), PAS Docker
- **Cargo workspace pour le code** → monorepo Rust, un `Cargo.lock`, deps partagées

---

## Structure du repository

```
reseau-racine/
├── Cargo.toml                    # Workspace root (resolver = "2")
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
│   │       ├── crypto.rs         # NIP-44, secp256k1, X25519
│   │       ├── identity.rs       # Génération/stockage de clés
│   │       ├── message.rs        # NIP-17: rumor → seal → gift wrap
│   │       └── transport/        # Abstraction multi-transport
│   │           ├── mod.rs
│   │           ├── nostr.rs      # Transport internet (WebSocket)
│   │           ├── reticulum.rs  # Transport local (Reticulum)
│   │           └── meshtastic.rs # Transport extrême (Meshtastic)
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
│       └── icons/                # Icônes de l'app
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
│   └── integration/              # Tests d'intégration (relais local)
├── scripts/
│   ├── setup.sh                  # Setup dev (installe deps système)
│   ├── test-e2e.sh               # Lance les tests E2E
│   └── release.sh                # Script de release (bump version + tag)
├── AGENTS.md                     # Instructions pour les agents IA
├── CONTRIBUTING.md               # Guide de contribution
└── README.md                     # Documentation principale
```

---

## Ce que chaque composant fait

### `rr-core` (bibliothèque)

| Module | Rôle | Dépendances |
|--------|------|------------|
| `crypto` | NIP-44 V2 (ChaCha20-Poly1305), secp256k1, X25519 | `nostr` crate, `x25519-dalek`, `chacha20poly1305` |
| `identity` | Génération/stockage de clés secp256k1, nsec/npub | `nostr` crate, `zeroize` |
| `message` | NIP-17: rumor → seal → gift wrap, unwrap | `nostr` crate, `serde_json` |
| `transport::nostr` | WebSocket vers relais Nostr, publish, subscribe | `tokio-tungstenite`, `serde_json` |
| `transport::reticulum` | Communication via Reticulum (subprocess ou FFI) | `tokio`, `serde` |
| `transport::meshtastic` | Communication via Meshtastic (API HTTP) | `reqwest`, `serde` |

### `rr-cli` (binaire CLI — POC)

| Commande | Rôle |
|----------|------|
| `rr init` | Génère une identité secp256k1, stocke dans `~/.rr/keys.json` |
| `rr add-contact <npub> <nom>` | Ajoute un contact au carnet local |
| `rr send <nom> "message"` | Envoie un message NIP-17 (gift wrap) sur le relais |
| `rr sync` | Reçoit les nouveaux messages, unwrap, affiche |
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
      - ipfs-data:/data/ipfs

volumes:
  ipfs-data:
```

### `Dockerfile` (DevContainer)

```dockerfile
# Image officielle DevContainer Rust — outils pré-installés
FROM mcr.microsoft.com/devcontainers/rust:1-bookworm

# Dépendances système pour Tauri v2 + libsodium
RUN apt-get update && apt-get install -y \
    libssl-dev \
    libgtk-3-dev \
    libwebkit2gtk-4.1-dev \
    libappindicator3-dev \
    librsvg2-dev \
    patchelf \
    libsodium-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Install Tauri CLI
RUN cargo install tauri-cli --locked

WORKDIR /workspace
```

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
      - run: cargo test --workspace --verbose

  build-cli:
    name: Build CLI (Linux)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo build --package rr-cli --release
      - uses: actions/upload-artifact@v4
        with:
          name: rr-cli-linux
          path: target/release/rr

  build-tauri:
    name: Build Tauri (Linux)
    runs-on: ubuntu-22.04
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
          - platform: ubuntu-22.04
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
        if: matrix.platform == 'ubuntu-22.04'
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

## Docker — Services d'infrastructure

### Ce qui tourne dans Docker

| Service | Image | Usage |
|---------|-------|-------|
| **nostr-rs-relay** | `scsibug/nostr-rs-relay` | Relais Nostr local pour tests et production |
| **IPFS (kubo)** | `ipfs/kubo` | Pinning de contenu, mirror d'articles |
| **PeerTube** | `chocobozzz/peertube` | Hébergement vidéo (nœud créateur) |
| **Owncast** | `owncast/owncast` | Streaming live (nœud créateur) |

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
| **`cargo fmt --check`** passe | Code formaté |
| **`cargo clippy`** passe | Pas de warnings |
| **DevContainer** se lance | `docker compose up -d` dans `.devcontainer/` |
| **CI green** sur push | GitHub Actions: lint + test + build |
| **POC CLI** fonctionne | `rr init` → `rr send` → `rr sync` entre 2 terminaux |
| **README** documente le setup | Un novice peut suivre les étapes |

---

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

1. **EPIC 1 — POC "Premier Message Chiffré"** (12h) — Alice et Bob échangent un message via NIP-17
2. **EPIC 2 — Groupes & Cellules** — Clés de groupe X25519, cellules de 3
3. **EPIC 3 — Forward Secrecy** — Double Ratchet au-dessus de NIP-44
4. **EPIC 4 — Reticulum WiFi** — Second transport, bascule automatique
5. **EPIC 5 — Client Tauri** — UI React, chat, contacts, paramètres
6. **EPIC 6 — Nœud Relais** — Pi 5 + Docker Compose + cache + IPFS
