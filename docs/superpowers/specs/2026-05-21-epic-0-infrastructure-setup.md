# EPIC 0 — Infrastructure & Setup : "Fondations"

> **Statut** : ✅ LIVRÉ (2026-05-22)
> **Objectif** : mettre en place tout ce qu'il faut pour que n'importe qui puisse cloner le repo, développer, tester, et build — sans rien installer de complexe sur sa machine.

---

## Principe

- ✅ **Docker pour les services** (Nostr relay) → `.devcontainer/compose.yaml`
- ✅ **DevContainer pour les devs** → environnement reproductible via compose
- ⏳ **Binaire natif pour les utilisateurs** → Tauri (squelette, pas de bundle EPIC 0)
- ✅ **Cargo workspace pour le code** → monorepo Rust, un `Cargo.lock`, deps partagées

---

## Structure du repository

```
reseau-racine/
├── Cargo.toml                    # Workspace root (resolver = "2")
├── Cargo.lock                    # Dépendances verrouillées
├── deny.toml                     # cargo-deny: advisories, licences, bans
├── rust-toolchain.toml           # Rust stable (autodétecté)
├── AGENTS.md                     # Instructions pour les agents IA
├── README.md                     # Documentation principale (pédagogique)
├── LICENSE                       # AGPL-3.0-or-later
│
├── .github/
│   ├── dependabot.yml            # MAJ hebdo cargo + actions
│   └── workflows/
│       ├── ci.yml                # lint + test + audit + build + cross-check
│       └── release.yml           # Build CLI sur tag v*
│
├── .devcontainer/
│   ├── devcontainer.json         # Config DevContainer (Docker Compose)
│   ├── Dockerfile                # Rust + GTK + libsodium + Node LTS
│   └── compose.yaml              # dev + nostr-relay (scsibug) services
│
├── docker/
│   └── nostr-relay/
│       └── config.toml           # Relais local sur ws://localhost:8080
│
├── crates/
│   ├── rr-core/                  # Bibliothèque fondamentale ✅
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── crypto.rs         # NIP-44 V2 (ChaCha20-Poly1305)
│   │       ├── identity.rs       # secp256k1, nsec/npub, seed BIP-39
│   │       ├── message.rs        # NIP-17: send_private_msg, unwrap_gift_wrap
│   │       └── transport/
│   │           ├── mod.rs        # Trait TransportProvider
│   │           └── nostr.rs      # Connexion WebSocket aux relais
│   │
│   ├── rr-cli/                   # CLI POC ✅
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs           # init, identity, add-contact, contacts, send, sync, restore
│   │
│   └── rr-tauri/                 # Squelette ⏳
│       ├── Cargo.toml
│       ├── build.rs
│       └── src/
│           ├── main.rs
│           └── lib.rs
│
├── scripts/
│   ├── dev.sh                    # Wrapper Docker : tout cargo dans le container
│   ├── cargo                     # Alias transparent cargo → dev.sh
│   ├── build.sh                  # Build workspace
│   ├── check.sh                  # Check + clippy + test
│   └── release.sh                # Build release
│
└── docs/
    └── superpowers/specs/        # Specs et EPICs
```

---

## Rétrospective — ce qui a changé par rapport au plan initial

| Élément | Planifié | Livré | Delta |
|---------|----------|-------|-------|
| Docker IPFS | `docker/ipfs/` avec compose | Pas livré | Pas nécessaire avant EPIC 2 |
| `docs.yml` | Workflow docs.rs | Pas livré | Pas de docs.rs publique en Phase 0 |
| `ui/` React | Frontend Tauri complet | Pas livré | EPIC 4 |
| `tests/e2e/` | Tests end-to-end | Pas livré | EPIC 1 |
| `tests/nip44_vectors.json` | KAT officiels | Pas livré | EPIC 1 (quand on enverra des messages) |
| `CONTRIBUTING.md` | Guide contribution | Pas livré | Faible priorité |
| `tauri.conf.json` | Config Tauri complète | Pas livré | Bloqué par GTK système (EPIC 4) |
| `scripts/setup.sh` | Setup deps système | Remplacé par Docker | Meilleur — zéro install OS |
| `scripts/test-e2e.sh` | Tests E2E | Pas livré | EPIC 1 |
| `release.yml` Tauri | Build + sign Tauri | Version CLI seulement | Tauri bloqué par GTK |

**Décisions prises en cours de route :**

- Rust 1.85 → `stable` (1.95) : les dépendances Nostr exigeaient 1.86+
- `zeroize` retiré des dépendances : pas utilisé directement (nostr gère le zeroize en interne)
- `scripts/dev.sh` plutôt que `scripts/setup.sh` : tout passe par Docker, rien sur l'OS
- Pas de `#[cfg(windows)]` pour les permissions 0600 : `from_mode()` n'existe pas sur Windows

---

## Critères de succès — status

| Critère | Statut | Preuve |
|---------|--------|--------|
| **`git clone` + `cargo build`** fonctionne | ✅ | CI green sur Linux, macOS, Windows |
| **`cargo test --workspace`** passe | ✅ | 7 tests, 3 suites |
| **Tests KAT NIP-44** inclus | ⏳ | EPIC 1 |
| **`cargo fmt --check`** passe | ✅ | CI lint green |
| **`cargo clippy`** passe | ✅ | 0 warnings |
| **`cargo-deny`** passe | ✅ | advisories + licenses + bans + sources OK |
| **DevContainer** se lance | ✅ | `docker compose up` dans `.devcontainer/` |
| **CI green** sur push | ✅ | 6 jobs : lint, test, audit, cross-check, build |
| **POC CLI** fonctionne | ✅ | `rr init` → `rr identity` → `rr add-contact` |
| **README** documente le setup | ✅ | Guide pédagogique de 8 sections |

---

## Sécurité — audit final

| Catégorie | Statut | Détail |
|-----------|--------|--------|
| `unsafe` code | ✅ 0 occurrences | Aucun bloc unsafe |
| Secrets en dur | ✅ 0 | Aucune clé/secret dans le code |
| `unwrap()` production | ✅ 0 | `expect()` + gestion d'erreur |
| Debug leak SK | ✅ | `#[derive(Debug)]` → Debug custom (npub seulement) |
| Seed phrase | ✅ | Prompt confirmation avant affichage |
| Permissions fichier | ✅ | 0o600 sur Unix |
| `cargo-deny` | ✅ | Tous les checks passent |
| Zeroize mémoire | ⚠️ | Reporté EPIC 5 (Forward Secrecy) |
| Double Ratchet | ⚠️ | Reporté EPIC 5 (conditionnel) |

---

## Prochaines étapes

1. **EPIC 1 — POC "Premier Message Chiffré"** — Alice et Bob échangent un message via NIP-17
2. **EPIC 2 — Groupes & Cellules** — Clé de groupe X25519, cellules de 3
3. **EPIC 3 — Reticulum WiFi** — Second transport, bascule automatique
4. **EPIC 4 — Client Tauri** — UI React, chat, contacts, notifications
5. **EPIC 5 — Forward Secrecy** — Double Ratchet
6. **EPIC 6 — Nœud Relais** — Pi 5 + Docker Compose + cache + IPFS
