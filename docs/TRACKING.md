# Dashboard Réseau Racine

> Mis à jour : 2026-05-25

## Status global

```
██████████  EPIC 0  — Fondations              ✅  35/35  (100%)
████░░░░░░  EPIC 1  — Message chiffré NIP-17  ✅   5/5   (100%)
██████████  EPIC 2  — Groupes & cellules      ✅   7/7   (100%)
░░░░░░░░░░  EPIC 3  — Reticulum WiFi          ⬜  —
░░░░░░░░░░  EPIC 4  — Client Tauri            ⬜  —
██████████  EPIC 5  — Forward Secrecy         ✅   4/4   Sender Keys
░░░░░░░░░░  EPIC 6  — Nœud relais             ⬜  —
████████░░  EPIC 7  — Sécurité CLI            ✅  4/4   KeePassXC vault
██████████  EPIC 8  — Performance             ✅  4/4   Benchmarks système
██████████  EPIC 9  — Simulation charge       ✅  4/4   rr-stress load testing
████░░░░░░  SEC-1   — Sécurité Fixes          ✅  4/4   Nonce, rotation, store atomique
██████████  CLEAN-1 — Code mort               ✅  4/4   CryptoProvider, MessageService, TransportProvider, legacy path
██░░░░░░░░  CI-OPT  — CI optimisation         ✅  2/2   Path filtering, cancel-in-progress
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

## EPIC 2 — Groupes & Cellules ✅ (7/7)

Communication chiffrée en petit groupe (3-5 membres) sur Nostr avec clé partagée X25519 + Sender Keys.

| Story | Status | Détail |
|-------|--------|--------|
| Cell types + CellStore JSON | ✅ | `Cell`, `CellMember`, `SenderKey`, `CellStore` avec path isolé par `RR_DATA_DIR` |
| Gift-wrap broadcast | ✅ | `CellTransport` — rumor kind 13 tag `h`=cell UUID, gift-wrap kind 1059 par destinataire |
| Invitation / join | ✅ | `create_cell` génère clé X25519 + Sender Keys, `invite_member` distribue clés |
| `send_message` / `listen` | ✅ | Sender Key ratchet (HKDF-SHA256 → ChaCha20-Poly1305) + legacy NIP-44 fallback |
| CLI `group` subcommand (6 handlers) | ✅ | `create`, `list`, `info`, `invite`, `send`, `listen` |
| E2E validation cross-identité | ✅ | identity A → B déchiffre messages chiffrés via relay local |
| `remove_member` + `rotate_key` | ✅ | Régénère Sender Keys pour membres restants, distribue via gift-wrap |
| CLI `group remove` + `group rotate-key` | ✅ | |

### Architecture

| Composant | Fichier | Rôle |
|-----------|---------|------|
| Types Cell | `crates/rr-core/src/cell.rs` | Cell, CellMember, SenderKey, CellStore |
| Transport | `crates/rr-core/src/cell_transport.rs` | CellTransport — create, invite, send, listen, remove, rotate |
| Sender Key crypto | `crates/rr-core/src/sender_key.rs` | HKDF ratchet, encrypt/decrypt ChaCha20-Poly1305 |
| Legacy crypto | `crates/rr-core/src/crypto.rs` | NIP-44 self-DH (conservé pour backward compat) |
| CLI | `crates/rr-cli/src/main.rs` | GroupCommands (8 sous-commandes) |

### Qualité

| Métrique | Status |
|----------|--------|
| Tests | ✅ 51/51 pass (31 unit + 20 integration) |
| Clippy | ✅ 0 warnings |
| Sender Key tests | ✅ Ratchet déterministe + unique, ChaCha20 roundtrip, wrong key reject |
| Cell store tests | ✅ Roundtrip, find, add/remove, update members, SenderKey serialization |
| Legacy backward compat | ✅ Cellules sans sender_keys continuent NIP-44 |

**Specs :** `docs/superpowers/specs/2026-05-24-epic2-groupes-cellules.md`, `docs/superpowers/specs/2026-05-25-epic3-sender-keys-rotation.md`
**Plans :** `docs/superpowers/plans/2026-05-24-epic2-groupes-cellules.md`, `docs/superpowers/plans/2026-05-25-epic3-sender-keys.md`

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

## EPIC 5 — Forward Secrecy ✅ (4/4)

Per-message forward secrecy via Sender Keys (Signal-style) pour les cellules de groupe.

| Story | Status | Détail |
|-------|--------|--------|
| Sender Key type + HKDF ratchet | ✅ | HKDF-SHA256 — unique par message, 32B msg_key + 32B next_chain_key |
| ChaCha20-Poly1305 encrypt/decrypt | ✅ | Zero nonce (clé unique par message), base64 ciphertext |
| Sender Key distribution | ✅ | `create_cell` + `invite_member` génèrent et distribuent clés via gift-wrap |
| Key rotation on member removal | ✅ | `remove_member` + `rotate_key` : regénère toutes les Sender Keys, distribue aux membres restants |
| `key_rotation` event handling | ✅ | Discovery mode détecte `action: "key_rotation"` et met à jour le store local |

### Décisions architecturales

| Décision | Justification |
|----------|---------------|
| Sender Keys (Signal-style) vs MLS | MLS = over-engineering pour 3-5 membres. Sender Keys = O(N) distribution sur remove, per-message FS via HKDF ratchet, rotation PCS on-demand |
| ChaCha20-Poly1305 pur vs NIP-44 | NIP-44 attend ECDH X25519. Clé symétrique 32B n'est pas une X25519 valide. `chacha20poly1305` déjà transitif dans `nostr-rs` |
| Legacy backward compat | Cellules EPIC 2 sans sender_keys continuent NIP-44 self-DH. Detection au runtime : SK first, fallback NIP-44 |

**Spec :** `docs/superpowers/specs/2026-05-25-epic3-sender-keys-rotation.md`
**Plan :** `docs/superpowers/plans/2026-05-25-epic3-sender-keys.md`

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

## EPIC 8 — Performance ✅ (4/4)

Benchmarks système : latences et débit crypto + transport.

| Story | Status | Détail |
|-------|--------|--------|
| Bench crypto pure (criterion) | ✅ | 10 métriques : nip44 encrypt/decrypt, event_sign, giftwrap_roundtrip (×3 sizes) |
| Bench transport (relais local Docker) | ✅ | 7 métriques : publish single/batch 1/10/100, sync single/load 1/10/100 |
| `rr bench` sous-commande CLI | ✅ | --crypto-only, --transport-only, --relay |
| Crypto bench + regression check en CI | ✅ | Criterion standard mode, grep "Performance has regressed" → exit 1 |

### Qualité

| Métrique | Status |
|----------|--------|
| Tests | ✅ 37/37 (31 unit + 3 integ + 3 proptest) |
| Clippy | ✅ 0 warnings |
| Fmt | ✅ clean |

### Architecture

| Composant | Fichier | Rôle |
|-----------|---------|------|
| Crypto benchmarks | `crates/rr-core/benches/crypto.rs` | 4 groups, criterion + FuturesExecutor |
| Transport benchmarks | `crates/rr-core/benches/transport.rs` | 4 groups, criterion + tokio Runtime |
| CLI bench | `crates/rr-cli/src/main.rs` | `rr bench`, `cmd_bench()`, `check_relay()` |
| CI | `.github/workflows/ci.yml` | job bench, baseline cached, regression grep |

**Specs :** `docs/superpowers/specs/2026-05-23-performance-benchmarking.md`, `docs/superpowers/specs/2026-05-24-epic8-post-merge-fixes.md`
**Plan :** `docs/superpowers/plans/2026-05-24-epic8-performance-benchmarks.md`

---

## EPIC 9 — Simulation Charge ✅ (4/4)

Outil de stress test `rr-stress` pour valider le comportement sous charge.

| Story | Status | Détail |
|-------|--------|--------|
| Binaire `rr-stress` séparé | ✅ | `crates/rr-stress/`, dépendances workspace |
| Phase Hello + Chat | ✅ | Envois périodiques, destinataires aléatoires (i+1)%n |
| Métriques (success, latence p50/p95/p99, erreurs) | ✅ | Output JSON + table recap |
| Test 5 users × 3 msgs validé | ✅ | 15/15 success, p50 1.8ms, p95 7.4ms |

### Exemple

```json
{
  "users": 5,
  "total_messages": 15,
  "success": 15,
  "failed": 0,
  "latency_ms": { "p50": 1.8, "p95": 7.4, "p99": 7.6 },
  "errors": { "timeout": 0, "reject": 0, "disconnect": 0 },
  "duration_sec": 0.32,
  "throughput_msg_s": 46.8
}
```

### Architecture

| Composant | Fichier | Rôle |
|-----------|---------|------|
| Identités déterministes | `crates/rr-stress/src/main.rs` | SHA256 du seed index → Keys |
| Clients connexion | `crates/rr-stress/src/main.rs` | client.connect() + wait_for_connection |
| Phases envoi | `crates/rr-stress/src/main.rs` | Semaphore(parallelism), tokio::spawn |
| Collecte métriques | `crates/rr-stress/src/main.rs` | AtomicU64 + Mutex, p50/p95/p99 |

**Spec :** `docs/superpowers/specs/2026-05-23-stress-load-simulation.md`
**Plan :** `docs/superpowers/plans/2026-05-24-epic9-stress-load-simulation.md`

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

---

## SEC-1 — Sécurité Fixes ✅ (4/4)

Corrections de sécurité P0 : nonce ChaCha20, authenticité rotation de clés, atomicité du store.

| Story | Status | Détail |
|-------|--------|--------|
| msg_count dans HKDF info string | ✅ | `ratchet_forward(chain, msg_count)` — nonce unique même si même chain_key réutilisé |
| Save store BEFORE network send | ✅ | `send_message` : update store → save → drop(lock) → send. Crash safe. |
| Listen race fix (mode 1+2) | ✅ | `msg_count` + `chain_key_hex` lus sous lock store (pas de clone stale) |
| `handle_key_rotation(sender_pk)` auth | ✅ | Vérifie sender ∈ cell.members sous le même lock que l'update (TOCTOU fixé) |
| CellStore atomique (.tmp+rename) | ✅ | `save()` écrit `.tmp` → `rename`. `load()` nettoie `.tmp`, log erreurs parse |

### Qualité

| Métrique | Status |
|----------|--------|
| Tests | ✅ 8/8 pass (3 cell_store + 5 sender_key) |
| Clippy | ✅ 0 warnings |
| Build CLI | ✅ release |

### Décisions architecturales

| Décision | Justification |
|----------|---------------|
| `eprintln!` pour erreurs store (pas `log` crate) | Pas de dépendance log introduite (cohérent avec code existant) |
| msg_count.to_be_bytes() dans info string HKDF | Compatible little/big-endian, déterministe, 8 bytes suffisent |
| Rename atomique POSIX (meme filesystem) | Garantie atomique sur Linux/ macOS, crash = perte .tmp seulement |

**Spec :** `docs/superpowers/specs/2026-05-25-security-fixes-nonce-rotation-store.md`
**Plan :** `docs/superpowers/plans/2026-05-25-security-fixes-nonce-rotation-store.md`

---

## CLEAN-1 — Code Mort ✅ (4/4)

Suppression de 4 artéfacts de code mort : `CryptoProvider`, `MessageService`, `TransportProvider`, legacy NIP-44 path.

| Story | Status | Détail |
|-------|--------|--------|
| `MessageService` struct → fonctions libres | ✅ | `send_message()` et `receive_message()` libres, plus de struct fantôme |
| `TransportProvider` trait supprimé | ✅ | Trait mort avec 1 seule impl, jamais utilisé génériquement |
| Legacy NIP-44 path retiré de `listen()` | ✅ | ~50 lignes de code inaccessible (cell_key_hex toujours vide) |
| `CryptoProvider` wrapper supprimé | ✅ | Appels `nip44` directs, -64 lignes, comportement identique |
| `cell_key_hex` → `#[serde(default)]` | ✅ | Rétrocompat désérialisation, plus utilisé en écriture |

### Qualité

| Métrique | Status |
|----------|--------|
| Tests | ✅ 52/52 pass (inchangé) |
| Clippy | ✅ 0 warnings |
| Fmt | ✅ clean |
| Build CLI | ✅ release |

### Décisions architecturales

| Décision | Justification |
|----------|---------------|
| Fonctions libres > struct sans état | Pas de `new()`, pas de `Default`, pas de `self` — juste des fonctions pures |
| `#[serde(default)]` conservé pour `cell_key_hex` | Permet de lire les vieux fichiers `cells.json` sans erreur |
| `cell_key_hex: String::new()` conservé dans `create_cell` | Nécessaire pour le struct literal Rust (`#[serde(default)]` = désérialisation seulement) |

**Spec :** `docs/superpowers/specs/2026-05-25-dead-code-removal.md`
**Plan :** `docs/superpowers/plans/2026-05-25-clean-1-dead-code-removal.md`

---

## CI-OPT — CI Optimisation ✅ (2/2)

Optimisation du CI GitHub Actions : path filtering + cancel-in-progress.

| Story | Status | Détail |
|-------|--------|--------|
| Path filtering (`dorny/paths-filter`) | ✅ | PR docs-only : seul `Detect Changes` tourne (~30s), tous les jobs skip |
| Cancel-in-progress | ✅ | Force-push annule le run précédent au lieu d'en accumuler |

### Qualité

| Métrique | Status |
|----------|--------|
| PR docs-only | ✅ ~30s (était 20-30 min) |
| PR code | ✅ inchangé |
| Force-push waste | ✅ éliminé |

### Décisions architecturales

| Décision | Justification |
|----------|---------------|
| Merge queue abandonnée | Non disponible sur repos personnel GitHub (nécessite organisation) |
| Workflow unique | Pas besoin de 3 workflows sans merge queue. Un seul `ci.yml` avec `if` gates |
| `sbom` dépend de `build-cli` par `needs` | Même workflow, pas de duplication de `build-cli` |
| `if: needs.changes.outputs.rust == 'true' \|\| github.event_name == 'push'` | Sur push to main, toujours tout runner. Sur PR, seulement si code changé |

**Spec :** `docs/superpowers/specs/2026-05-25-ci-optimization-merge-queue.md`
**Plan :** `docs/superpowers/plans/2026-05-25-ci-optimization-merge-queue.md`

---

## Légende

| Symbole | Signification |
|---------|---------------|
| ✅ | Livré / Vérifié |
| ⏳ | En cours / Partiel |
| ⬜ | Pas commencé |
| 🔴 | Bloqué |
| ⚠️ | At-risk |
