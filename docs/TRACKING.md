# 🗺️ Dashboard Réseau Racine

> Mis à jour : 2026-05-22

## Status global

```
████████░░  EPIC 0  — Fondations              ✅  24/24  (100%)
░░░░░░░░░░  EPIC 1  — Premier message chiffré ⏳   0/?      (0%)
░░░░░░░░░░  EPIC 2  — Groupes & cellules      ⬜  —
░░░░░░░░░░  EPIC 3  — Reticulum WiFi          ⬜  —
░░░░░░░░░░  EPIC 4  — Client Tauri            ⬜  —
░░░░░░░░░░  EPIC 5  — Forward Secrecy         ⬜  —
░░░░░░░░░░  EPIC 6  — Nœud relais             ⬜  —
```

---

## EPIC 0 — Fondations ✅ (100%)

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
| Pre-commit hook `.githooks/pre-commit` (pas de rhusky) | ✅ |
| Makefile (build, test, fmt, lint, audit, ci, hooks) | ✅ |
| Templates GitHub + EditorConfig + VSCode + SECURITY.md | ✅ |
| **Tests (cargo test --workspace)** | ✅ 7/7 pass |
| **Clippy** | ✅ 0 warnings |
| **cargo-deny** | ✅ 4/4 OK |
| **Fuzzing (NIP-44 roundtrip + decrypt invalid + identity parse)** | ✅ |
| **cargo-udeps (unused dependencies CI)** | ✅ |
| **cargo auditable (binary-level audit)** | ✅ |

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
