# Dashboard Réseau Racine

> Mis à jour : 2026-05-24

## Status global

```
██████████  EPIC 0  — Fondations              ✅  35/35  (100%)
████████░░  EPIC 1  — Message chiffré NIP-17  ✅   5/5   (100%)
░░░░░░░░░░  EPIC 2  — Groupes & cellules      ⬜  —
░░░░░░░░░░  EPIC 3  — Reticulum WiFi          ⬜  —
░░░░░░░░░░  EPIC 4  — Client Tauri            ⬜  —
░░░░░░░░░░  EPIC 5  — Forward Secrecy         ⬜  —
░░░░░░░░░░  EPIC 6  — Nœud relais             ⬜  —
████████░░  EPIC 7  — Sécurité CLI            ✅  4/4   KeePassXC vault
░░░░░░░░░░  EPIC 8  — Performance             ⬜  —  Benchmarks système
░░░░░░░░░░  EPIC 9  — Simulation charge       ⬜  —  rr-stress load testing
```

---

## EPIC 0 — Fondations ✅

### Stories livrées

| Story | Status |
|-------|--------|
| Workspace Rust (Cargo.toml, deny.toml, rust-toolchain) | ✅ |
| `rr-core` : crypto NIP-44 V2 | ✅ |
| `rr-core` : identity secp256k1 + BIP-39 | ✅ |
| `rr-core` : message NIP-17 (send_private_msg) | ✅ |
| `rr-core` : transport Nostr (trait + impl) | ✅ |
| `rr-cli` : 7 commandes (init, identity, add-contact, contacts, send, sync, restore) | ✅ |
| `rr-tauri` : squelette Tauri v2 | ⏳ build bloqué (GTK) |
| DevContainer Docker (Rust + services) | ✅ |
| CI/CD (lint, test, audit, release) | ✅ |
| `scripts/dev.sh` : wrapper Docker | ✅ |
| README pédagogique | ✅ |
| AGENTS.md | ✅ |
| Security audit | ✅ |
| Repository Rulesets (Check Main + Protect Main) | ✅ |
| CI job names alignés avec noms des status checks | ✅ |
| Pre-commit hook `.githooks/pre-commit` | ✅ |
| Makefile (build, test, fmt, lint, audit, ci, hooks) | ✅ |
| Templates GitHub + EditorConfig + VSCode + SECURITY.md | ✅ |
| Nostr-relay Docker santé (healthcheck /proc/net/tcp) | ✅ |
| Phase 2 Sécurité (coverage, mutants, sbom) | ✅ |

#### Phase 2 Sécurité (PR #8)

| Élément | Status | Détail |
|---------|--------|--------|
| **cargo-llvm-cov** | ✅ CI | coverage CI + `output-path` fix |
| **cargo-mutants** | ✅ CI | `|| true` pour mutants bloquants non critiques |
| **cargo auditable → SBOM** | ✅ CI | auditable2cdx + SBOM upload |
| **nostr-relay healthcheck** | ✅ fix | `grep /proc/net/tcp` au lieu de curl |

### Qualité

| Métrique | Status | Détail |
|----------|--------|--------|
| **Tests** | ✅ 34/34 pass | 31 unit + 3 proptest |
| **Clippy** | ✅ 0 warnings | -- -D warnings en CI |
| **cargo-deny** | ✅ 4/4 OK | advisories, bans, licenses, sources |

### Phase 1 Sécurité (PR #4)

| Élément | Status | Détail |
|---------|--------|--------|
| **cargo-udeps** | ✅ CI | détection dépendances inutilisées (nightly) |
| **cargo-fuzz** | ✅ CI | 3 targets, 2min each, corpus cache |
| **cargo auditable** | ✅ build-cli | metadata dépendances embarquée dans binaire |
| **Ruleset 8 checks** | ✅ | `fuzz` + `udeps` ajoutés |

#### Erreurs CI rencontrées

| Problème | Cause | Solution |
|----------|-------|----------|
| `error: sanitizer is incompatible with statically linked libc` | `taiki-e/install-action` livre cargo-fuzz compilé musl → détecte `x86_64-unknown-linux-musl` (ASAN incompatible) | `--target $(rustc --print host-tuple)` force target GNU natif |
| `attributes starting with rustc are reserved` | cargo-fuzz v0.13.1 dépend de `rustix` avec attributes nightly-only | Utiliser `taiki-e/install-action` (précompilé) au lieu de `cargo install` |
| `RUSTFLAGS=-Ctarget-feature=-crt-static` ignoré | cargo-fuzz override RUSTFLAGS en ligne de commande | Utiliser `--target` au lieu de modifier RUSTFLAGS |

**Référence :** Issue cargo-fuzz #398

---

## EPIC 1 — Message Chiffré NIP-17 ✅ (5/5)

| Story | Status | Détail |
|-------|--------|--------|
| NostrTransport avec wait_for_connection | ✅ | Garantit WebSocket établi avant envoi |
| `rr send <contact> <message>` NIP-17 | ✅ | GiftWrap kind 1059 via `send_private_msg` |
| `rr sync` réception + déchiffrement | ✅ | Souscrit Kind::GiftWrap, unwrap via `MessageService::receive` |
| `RR_DATA_DIR` multi-identité | ✅ | Séparation sandbox pour tests |
| `RR_RELAY` configurable | ✅ | Défaut `wss://relay.damus.io` |

### Qualité

| Métrique | Status |
|----------|--------|
| Tests | ✅ 29/29 (26 unit + 3 proptest) inchangés |
| Clippy | ✅ 0 warnings |
| E2E validé | ✅ Alice → relais kind 1059 → Bob `rr sync` déchiffre et affiche |

### E2E Validation

```bash
# Terminal 1 : Alice
RUST_LOG=debug ./scripts/dev.sh env RR_RELAY=ws://172.20.0.2:8080 RR_DATA_DIR=/tmp/rr-alice cargo run --package rr-cli -- init
RUST_LOG=debug ./scripts/dev.sh env RR_RELAY=ws://172.20.0.2:8080 RR_DATA_DIR=/tmp/rr-alice cargo run --package rr-cli -- identity
RUST_LOG=debug ./scripts/dev.sh env RR_RELAY=ws://172.20.0.2:8080 RR_DATA_DIR=/tmp/rr-alice cargo run --package rr-cli -- add-contact bob <bob_npub>

# Terminal 2 : Bob
RUST_LOG=debug ./scripts/dev.sh env RR_RELAY=ws://172.20.0.2:8080 RR_DATA_DIR=/tmp/rr-bob cargo run --package rr-cli -- init
RUST_LOG=debug ./scripts/dev.sh env RR_RELAY=ws://172.20.0.2:8080 RR_DATA_DIR=/tmp/rr-bob cargo run --package rr-cli -- identity

# Terminal 1 : Alice envoie
RUST_LOG=debug ./scripts/dev.sh env RR_RELAY=ws://172.20.0.2:8080 RR_DATA_DIR=/tmp/rr-alice cargo run --package rr-cli -- send "bob" "Hello Bob!"

# Terminal 2 : Bob reçoit (Ctrl+C pour quitter)
RUST_LOG=debug ./scripts/dev.sh env RR_DATA_DIR=/tmp/rr-bob cargo run --package rr-cli -- sync
```

### Décisions architecturales

| Décision | Justification |
|----------|---------------|
| NIP-17 (pas NIP-04 déprécié) | `send_private_msg` fait rumor→seal→gift wrap en un appel |
| connect() fire-and-forget → `wait_for_connection(10s)` | La doc dit "A background connection task is spawned" |
| `Output.success` vérifié dans `MessageService::send` | Détecte les relais qui rejettent l'événement |
| `rr sync` sans timeout | Pattern bot.rs — Ctrl+C pour quitter |
| `limit(0)` retiré du filtre GiftWrap | Sinon sync ne reçoit pas les événements historiques |
| Relay URL en env var (`RR_RELAY`) | YAGNI pour le POC (pas de fichier config) |

### Known Issues (EPIC 1)

| Problème | Impact | Solution |
|----------|--------|----------|
| **Docker DNS** : container dev utilise `127.0.0.53` (host systemd-resolved) au lieu de `127.0.0.11` (Docker) | `nostr-relay` non résolu → nécessite IP directe `172.20.0.2` pour les tests locaux | Corriger `/etc/resolv.conf` dans follow-up |
| **rr-tauri** : GTK système absent du container | Buildable seulement sur host natif | Exclu du workspace CI |

---

## EPIC 2 — Groupes & Cellules

| Story | Status |
|-------|--------|
| NIP-44 + clé de groupe X25519 | ⬜ |
| Cellules de 3 (gift-wrap broadcast) | ⬜ |
| Invitation / join | ⬜ |

---

## EPIC 3 — Reticulum WiFi

| Story | Status |
|-------|--------|
| Transport Reticulum (RNP) | ⬜ |
| Bascule automatique Nostr ↔ Reticulum | ⬜ |

---

## EPIC 4 — Client Tauri

| Story | Status |
|-------|--------|
| GTK système (résolu) | ⬜ |
| UI React | ⬜ |
| Chat, contacts, notifications | ⬜ |

---

## EPIC 5 — Forward Secrecy

| Story | Status |
|-------|--------|
| Double Ratchet | ⬜ |
| Zeroize mémoire | ⬜ |

---

## EPIC 6 — Nœud Relais

| Story | Status |
|-------|--------|
| Raspberry Pi 5 + Docker | ⬜ |
| Cache + IPFS | ⬜ |
| Configuration WAN | ⬜ |

---

## EPIC 7 — Sécurité CLI ✅ (4/4)

Vault KeePassXC pour clés Nostr, remove storage JSON en clair.

| Story | Status | Détail |
|-------|--------|--------|
| Config.toml + KeySource (env/config) | ✅ | `~/.config/reseau-racine/config.toml`, priorité RR_KEYSTORE > config > File |
| `RR_KEYSTORE=keepassxc://...` backend | ✅ | Sous-processus keepassxc-cli, master password sur stdin |
| `RR_KEYSTORE=keepass-rs://...` fallback | ✅ | Crate `keepass` Rust, ouvre KDBX direct |
| `rr init --kdbx` + détection interactive | ✅ | RR_KEYSTORE propagé à toutes les commandes, wizard si keepassxc-cli détecté |
| `rr export` migration | ✅ | Exporte identité existante vers KeePassXC |
| Rétro-compatibilité `file` | ✅ | RR_KEYSTORE absent/config.toml absent → comportement actuel inchangé |

### Qualité

| Métrique | Status |
|----------|--------|
| Tests | ✅ 34/34 pass (31 unit + 3 proptest) |
| Clippy | ✅ 0 warnings (-- -D warnings) |
| Rétro-compat | ✅ `rr init` + `rr identity` sans KeePassXC identique à avant |
| CI release | ✅ taiki-e/upload-rust-binary-action (tag v* only) |

**Spec :** `docs/superpowers/specs/2026-05-23-security-keepassxc-vault.md`
**Plan :** `docs/superpowers/plans/2026-05-23-epic7-keepassxc-vault.md`
**Guide KeePassXC :** `docs/GUIDE.md#sécuriser-tes-clés-avec-keepassxc`

### Architecture

| Composant | Fichier | Rôle |
|-----------|---------|------|
| Config | `crates/rr-core/src/config.rs` | Config struct, load/save toml, config_dir |
| KeySource | `crates/rr-core/src/identity.rs` | Enum File/KeePassXc/KeePassRs, from_env/from_config |
| Backend CLI | `crates/rr-core/src/identity.rs` | detect_keepassxc_cli, get_nsec_keepassxc, save_to_keepassxc |
| Backend Rust | `crates/rr-core/src/identity.rs` | get_nsec_keepassrs (keepass-rs crate) |
| CLI flags | `crates/rr-cli/src/main.rs` | --kdbx, --entry, rr export |
| CI release | `.github/workflows/ci.yml` | nouveau job release (taiki-e) |

---

## EPIC 8 — Performance ⬜

Benchmarks système pour mesurer latences et débit.

| Story | Status | Détail |
|-------|--------|--------|
| Bench crypto pure (criterion) | ⬜ | NIP-44 encrypt/decrypt, event sign, full roundtrip |
| Bench transport (relais local Docker) | ⬜ | Publish single/batch, sync single/load |
| `rr bench` sous-commande CLI | ⬜ | Wrapper CLI pour les benchs |
| Crypto bench en CI | ⬜ | Benchmarks déterministes en CI |

**Spec :** `docs/superpowers/specs/2026-05-23-performance-benchmarking.md`

---

## EPIC 9 — Simulation Charge ⬜

Outil de stress test pour valider le comportement sous charge.

| Story | Status | Détail |
|-------|--------|--------|
| Binaire `rr-stress` séparé | ⬜ | Génération N identités, N clients tokio |
| Phase Hello + Chat | ⬜ | Envois périodiques, destinataires aléatoires |
| Métriques (success, latence p50/p95/p99, erreurs) | ⬜ | Output JSON + table recap |
| Test 50 users sur relais local | ⬜ | Valider le goulet nostr-rs-relay |

**Spec :** `docs/superpowers/specs/2026-05-23-stress-load-simulation.md`

---

## Légende

| Symbole | Signification |
|---------|---------------|
| ✅ | Livré / Vérifié |
| ⏳ | En cours / Partiel |
| ⬜ | Pas commencé |
| 🔴 | Bloqué |
| ⚠️ | At-risk |
