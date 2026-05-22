# Dashboard Réseau Racine

> Mis à jour : 2026-05-22

## Status global

```
████████░░  EPIC 0  — Fondations              ✅  27/27  (100%)
░░░░░░░░░░  EPIC 1  — Premier message chiffré ⏳   0/?      (0%)
░░░░░░░░░░  EPIC 2  — Groupes & cellules      ⬜  —
░░░░░░░░░░  EPIC 3  — Reticulum WiFi          ⬜  —
░░░░░░░░░░  EPIC 4  — Client Tauri            ⬜  —
░░░░░░░░░░  EPIC 5  — Forward Secrecy         ⬜  —
░░░░░░░░░░  EPIC 6  — Nœud relais             ⬜  —
```

---

## EPIC 0 — Fondations ✅ (27/27)

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

### Qualité

| Métrique | Status | Détail |
|----------|--------|--------|
| **Tests** | ✅ 29/29 pass | 4 suites (unit + proptest + doc + binary) |
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

**Référence :** Issue cargo-fuzz #398, confirmé par kevinburkesegment/coreutils fuzzing.yml

### Détails commandes fuzz

```bash
# Build
cargo +nightly fuzz build --target $(rustc --print host-tuple)

# Run (exemple roundtrip, 10s)
cargo +nightly fuzz run fuzz_nip44_roundtrip --target $(rustc --print host-tuple) -- -max_total_time=10 -runs=10000
```

### Architecture fuzz

```
crates/rr-core/fuzz/
├── .gitignore           # ignore target/
├── Cargo.lock
├── Cargo.toml           # standalone workspace (pas dans workspace root)
├── fuzz_targets/
│   ├── fuzz_nip44_roundtrip.rs   # encrypt→decrypt roundtrip
│   ├── fuzz_nip44_decrypt.rs     # decrypt sans panique
│   └── fuzz_identity_parse.rs    # parsing nsec/npub/mnemonic
└── corpus/              # cache CI
```

**Prochaine étape :** EPIC 1 — connecter la CLI à `nostr-relay:8080` et envoyer un message réel

---

## EPIC 1 — Premier Message Chiffré ⏳ (0%)

| Story | Status |
|-------|--------|
| Client Nostr connecté au relais local | ⬜ |
| `rr send` envoie un vrai message NIP-17 | ⬜ |
| `rr sync` reçoit et déchiffre | ⬜ |
| Tests round-trip Alice ↔ Bob | ⬜ |

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

## Légende

| Symbole | Signification |
|---------|---------------|
| ✅ | Livré / Vérifié |
| ⏳ | En cours / Partiel |
| ⬜ | Pas commencé |
| 🔴 | Bloqué |
| ⚠️ | At-risk |
