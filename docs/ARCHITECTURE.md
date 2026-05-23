# Architecture Réseau Racine

## Status EPICs

| EPIC | Status | Description |
|------|--------|-------------|
| 0 | ✅ | Fondations : CI/CD, DevContainer, sécurité, qualité |
| 1 | ✅ | Message chiffré NIP-17 (GiftWrap) |
| 2 | ⬜ | Groupes & cellules |
| 3 | ⬜ | Reticulum WiFi |
| 4 | 🔴 | Client Tauri (bloqué GTK) |
| 5 | ⬜ | Forward Secrecy |
| 6 | ⬜ | Nœud relais |
| 7 | ⬜ | Sécurité CLI : vault KeePassXC |
| 8 | ⬜ | Performance : benchmarks |
| 9 | ⬜ | Simulation charge : rr-stress |

## 1. Vue d'ensemble

```mermaid
graph TB
    subgraph CLI["rr-cli (binary)"]
        INIT["rr init"]
        IDENTITY["rr identity"]
        CONTACTS["rr add-contact / contacts"]
        SEND["rr send"]
        SYNC["rr sync"]
        RESTORE["rr restore"]
        BENCH["rr bench ⬜"]
    end

    subgraph CORE["rr-core (library)"]
        CRYPTO["NIP-44 V2<br/>encrypt / decrypt"]
        ID["IdentityManager<br/>secp256k1 + BIP-39"]
        MSG["MessageService<br/>send / receive"]
        TRANS["NostrTransport<br/>connect / wait_for_connection"]
        KEYSTORE["KeySource ⬜<br/>file / keepassxc / keepass-rs"]
    end

    subgraph STRESS["rr-stress (binary) ⬜"]
        LOAD["Load simulation<br/>N users, N messages"]
        METRICS["Métriques<br/>p50/p95/p99/success"]
    end

    subgraph FUTURE["Planned crates"]
        TAURI["rr-tauri 🔴<br/>Tauri v2 (GTK bloqué)"]
        RELAY_NODE["rr-relay ⬜<br/>Nœud relais embarqué"]
    end

    SEND --> MSG
    SYNC --> MSG
    MSG --> CRYPTO
    MSG --> TRANS
    SEND --> KEYSTORE
    IDENTITY --> ID
    CLI --> CORE
    BENCH --> CORE
    STRESS --> CORE
    STRESS --> DOCKER_RELAY
    TRANS --> DOCKER_RELAY["nostr-relay Docker<br/>ws://172.20.0.2:8080"]
    MSG --> DOCKER_RELAY
```

## 2. CLI et routage

```mermaid
graph TD
    RR["rr (clap)"]
    INIT["rr init<br/>→ génère identité BIP-39"]
    ID_CMD["rr identity<br/>→ affiche npub/nsec"]
    ADD["rr add-contact <alias> <npub><br/>→ enregistre contact"]
    LS["rr contacts<br/>→ liste contacts"]
    SEND_CMD["rr send <alias> <message><br/>→ NIP-17 GiftWrap"]
    SYNC_CMD["rr sync<br/>→ subscribe kind 1059 en temps réel"]
    RESTORE_CMD["rr restore <mnemonic><br/>→ restaure identité BIP-39"]
    BENCH_CMD["rr bench ⬜<br/>→ benchmarks système"]

    RR --> INIT
    RR --> ID_CMD
    RR --> ADD
    RR --> LS
    RR --> SEND_CMD
    RR --> SYNC_CMD
    RR --> RESTORE_CMD
    RR --> BENCH_CMD
```

## 3. DevContainer

```mermaid
graph LR
    subgraph HOST["Host"]
        SCRIPTS["scripts/dev.sh"]
        MAKE["Makefile"]
    end

    subgraph DOCKER["Docker Compose"]
        DEV["dev<br/>Rust 1.95.0<br/>opencode"]
        RELAY["nostr-relay<br/>ws://172.20.0.2:8080<br/>healthcheck: /proc/net/tcp"]
    end

    SCRIPTS --> DEV
    SCRIPTS --> RELAY
    DEV --> RELAY
```

Le container `dev` utilise `127.0.0.53` (systemd-resolved host) au lieu de `127.0.0.11` (DNS Docker) → `nostr-relay` non résolu par nom. Workaround : utiliser l'IP directe `172.20.0.2`.

## 4. CI/CD Pipeline (13 jobs)

```mermaid
graph TD
    subgraph QUALITY["Quality (3)"]
        LINT["lint<br/>fmt + clippy"]
        TEST["test<br/>29 tests"]
        AUDIT["audit<br/>cargo-deny 4/4"]
    end

    subgraph CROSS["Cross-platform (2)"]
        MAC["check-cross macos-latest"]
        WIN["check-cross windows-latest"]
    end

    subgraph BUILD["Build (1)"]
        CLI["build-cli<br/>release + auditable"]
    end

    subgraph SEC["Security (3)"]
        FUZZ["fuzz × 3 targets<br/>NIP-44 roundtrip/decrypt<br/>identity parse"]
        UDEPS["udeps<br/>nightly unused deps"]
        MUTANTS["mutants<br/>test mutation"]
    end

    subgraph METRICS["Metrics (2)"]
        COV["coverage<br/>cargo-llvm-cov"]
        SBOM["sbom<br/>auditable2cdx + upload"]
    end

    subgraph CHECKS["Required checks (8)"]
        R1["lint"] R2["test"] R3["audit"]
        R4["fuzz"] R5["udeps"]
        R6["check-cross (macos)"] R7["check-cross (windows)"]
        R8["build-cli"]
    end

    LINT --> R1
    TEST --> R2
    AUDIT --> R3
    FUZZ --> R4
    UDEPS --> R5
    MAC --> R6
    WIN --> R7
    CLI --> R8
```

## 5. Flux message NIP-17 ✅ (EPIC 1)

```mermaid
sequenceDiagram
    participant A as Alice (rr send)
    participant RELAY as nostr-relay
    participant B as Bob (rr sync)

    Note over A: KeySource (file) → lit nsec JSON
    A->>A: NIP-44 encrypt(content, bob_pubkey)
    A->>A: Wrap in rumor (kind 14)
    A->>A: Seal with alice_privkey (kind 13)
    A->>A: GiftWrap with ephemeral key (kind 1059)
    A->>RELAY: publish event kind 1059
    Note over RELAY: Client::send_private_msg()<br/>Vérifie Output.success

    B->>RELAY: subscribe filter: kind=1059, #p=bob_pubkey
    RELAY-->>B: event kind 1059
    B->>B: MessageService::receive(client, event)
    B->>B: Unwrap GiftWrap → seal → rumor
    B->>B: NIP-44 decrypt(content, alice_pubkey)
    Note over B: Affiche: 📨 alice: message<br/>Ctrl+C pour quitter
```

### Décisions architecturales clés

| Décision | Pourquoi |
|----------|----------|
| NIP-17 (pas NIP-04 déprécié) | `send_private_msg` fait rumor→seal→gift wrap en un appel |
| `connect()` fire-and-forget | Background task — nécessite `wait_for_connection(10s)` |
| `Output.success` vérifié | Détecte les relais qui rejettent l'événement |
| `rr sync` sans timeout | Ctrl+C pour quitter (pattern bot.rs) |
| RR_RELAY en env var | YAGNI POC (pas de fichier config) |

## 6. Identity lifecycle

```mermaid
flowchart LR
    subgraph INIT["rr init"]
        BIP39["BIP-39 mnemonic (12 words)"]
        SEED["Seed → secp256k1 keypair"]
        NSEC["nsec / npub"]
        JSON["save: identities/*.json<br/>nsec en clair"]
    end

    subgraph USE["rr identity / send / sync"]
        LOAD["Load from JSON<br/>ou KeePassXC ⬜"]
        SIGN["NIP-44 encrypt + NIP-17 sign"]
    end

    subgraph RESTORE["rr restore"]
        PHRASE["12-word mnemonic"]
        RESTORE_SEED["BIP-39 deterministic"]
        SAME["Même keypair (déterministe)"]
    end

    BIP39 --> SEED --> NSEC --> JSON
    JSON --> LOAD
    PHRASE --> RESTORE_SEED --> SAME
    LOAD --> SIGN
```

## 7. Vault KeePassXC ⬜ (EPIC 7)

```mermaid
flowchart TD
    CLI["rr send bob 'hello'"]
    ID["IdentityManager::load()"]
    RR_KEYSTORE{"RR_KEYSTORE ?"}
    FILE["file (défaut)"]
    XC["keepassxc://..." ]
    RS["keepass-rs://..."]
    JSON["~/.local/share/.../identities/*.json"]
    CLI_XC["keepassxc-cli show"]
    CLI_RS["keepass-rs crate"]
    MASTER["Prompt master password"]
    NSEC["nsec en mémoire"]
    SEND["NIP-17 GiftWrap → relais"]
    ZERO["zeroize après usage"]

    CLI --> ID
    ID --> RR_KEYSTORE
    RR_KEYSTORE -- absent/file --> FILE
    FILE --> JSON
    JSON --> NSEC
    RR_KEYSTORE -- keepassxc:// --> XC
    XC --> CLI_XC
    CLI_XC --> MASTER
    MASTER --> NSEC
    RR_KEYSTORE -- keepass-rs:// --> RS
    RS --> CLI_RS
    CLI_RS --> MASTER
    NSEC --> SEND
    SEND --> ZERO
```

## 8. Benchmarks ⬜ (EPIC 8)

```mermaid
flowchart LR
    subgraph CRYPTO_BENCH["Bench Crypto (criterion)"]
        ENC["NIP-44 encrypt 1KB"]
        DEC["NIP-44 decrypt"]
        SIGN["Event sign kind 1059"]
        GW["GiftWrap full roundtrip<br/>encrypt→seal→unwrap→decrypt"]
    end

    subgraph TRANSPORT_BENCH["Bench Transport (Docker)"]
        PUB1["Publish single<br/>connect→wait→publish"]
        PUBN["Publish batch<br/>N messages (1,10,100)"]
        SYNC1["Sync single<br/>subscribe→unwrap"]
        SYNCN["Sync load<br/>N messages→receive→unwrap"]
    end

    subgraph CLI_RUN["rr bench --count N"]
        CLI_CMD["rr bench --count 10"]
    end

    CLI_CMD --> CRYPTO_BENCH
    CLI_CMD --> TRANSPORT_BENCH

    PUB1 --> RELAY["nostr-relay Docker<br/>ws://172.20.0.2:8080"]
    PUBN --> RELAY
    SYNC1 --> RELAY
    SYNCN --> RELAY
```

## 9. Simulation charge ⬜ (EPIC 9)

```mermaid
flowchart TD
    SEED["Seed stable (index)"]
    GEN["Générer N identités déterministes"]
    CLIENTS["Créer N clients tokio<br/>Keys + Client nostr-sdk"]

    subgraph PHASE_HELLO["Phase Hello"]
        HELLO["Chaque user → 1 destinataire aléatoire<br/>Évite N×M broadcast"]
    end

    subgraph PHASE_CHAT["Phase Chat"]
        CHAT["Messages périodiques<br/>toutes les --interval ms"]
    end

    subgraph COLLECT["Collecte Métriques"]
        SUCCESS["success_count / total"]
        LATENCY["latence p50 / p95 / p99"]
        ERRORS["errors: timeout, reject, disconnect"]
    end

    subgraph OUTPUT["Output"]
        JSON["results/stress-*.json"]
        TABLE["Table récap console"]
    end

    SEED --> GEN --> CLIENTS
    CLIENTS --> PHASE_HELLO
    CLIENTS --> PHASE_CHAT
    PHASE_HELLO --> COLLECT
    PHASE_CHAT --> COLLECT
    COLLECT --> OUTPUT
    OUTPUT --> RELAY["nostr-relay ws://172.20.0.2:8080"]
```

## 10. Fuzz testing

```mermaid
graph TD
    subgraph CRATES["crates/rr-core/fuzz/"]
        T1["fuzz_nip44_roundtrip<br/>encrypt→decrypt == plaintext"]
        T2["fuzz_nip44_decrypt<br/>ne panique pas sur données invalides"]
        T3["fuzz_identity_parse<br/>nsec/npub/mnemonic malformés"]
    end

    subgraph CI["CI (2min each)"]
        BUILD["cargo +nightly fuzz build<br/>--target x86_64-gnu"]
        RUN["cargo +nightly fuzz run<br/>-- -max_total_time=120"]
        CACHE["Corpus cache<br/>entre les runs CI"]
    end

    subgraph INFRA["Infra"]
        TARGET["--target $(rustc --print host-tuple)<br/>évite le bug musl ASAN"]
        INSTALL["taiki-e/install-action@v2<br/>précompilé, pas cargo install"]
    end

    T1 --> BUILD
    T2 --> BUILD
    T3 --> BUILD
    BUILD --> RUN
    T1 --> CACHE
    T2 --> CACHE
    T3 --> CACHE
    INFRA --> BUILD
```

Cf. [cargo-fuzz issue #398](https://github.com/rust-fuzz/cargo-fuzz/issues/398)

## 11. Pre-commit hook

```mermaid
flowchart LR
    COMMIT["git commit"]
    FMT["cargo fmt --all --check"]
    CLIPPY["cargo clippy -- -D warnings"]
    TEST["cargo test --locked"]
    OK["✅ Commit accepté"]
    FAIL["❌ Commit bloqué"]

    COMMIT --> FMT
    FMT -- pass --> CLIPPY
    FMT -- fail --> FAIL
    CLIPPY -- pass --> TEST
    CLIPPY -- fail --> FAIL
    TEST -- pass --> OK
    TEST -- fail --> FAIL
```

Installation : `make hooks` ou `git config core.hooksPath .githooks`

## 12. Organisation fichiers

```
reseau-racine/
├── .devcontainer/
│   ├── compose.yaml          # dev + nostr-relay
│   ├── Dockerfile            # Rust 1.95.0 + outils
│   └── nostr-relay/
│       └── config.toml       # relay config
├── .githooks/
│   └── pre-commit            # fmt + clippy + test
├── .github/workflows/
│   └── ci.yml                # 13 jobs, 8 status checks
├── crates/
│   ├── rr-cli/
│   │   └── src/main.rs       # clap CLI routing
│   ├── rr-core/
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── identity.rs   # IdentityManager, KeySource
│   │   │   ├── message.rs    # MessageService (send/receive)
│   │   │   └── transport/
│   │   │       └── nostr.rs  # NostrTransport
│   │   └── fuzz/
│   │       └── fuzz_targets/ # 3 targets
│   ├── rr-stress/ ⬜          # Load simulation
│   └── rr-tauri/ 🔴           # Tauri v2 (GTK bloqué)
├── docs/
│   ├── ARCHITECTURE.md        # Ce fichier
│   ├── TRACKING.md            # Suivi des EPICs
│   └── superpowers/
│       └── specs/             # Specs approuvées
└── scripts/
    └── dev.sh                 # Wrapper Docker
```

## 13. Modules Rust et dépendances

```mermaid
graph RL
    subgraph RR_CLI["rr-cli"]
        MAIN["main.rs"]
    end

    subgraph RR_CORE["rr-core"]
        IDENTITY["identity.rs"]
        MESSAGE["message.rs"]
        TRANSPORT["transport/nostr.rs"]
    end

    subgraph EXTERNAL["Externe (Cargo.toml)"]
        NS["nostr-sdk 0.44"]
        N44["nostr::nips::nip44"]
        N59["nostr::nips::nip59"]
        KP["keepass-rs ⬜<br/>lecture KDBX"]
    end

    subgraph KEYCHAIN["KeePassXC"]
        KDBX["vault.kdbx"]
        CLI_XC_EXT["keepassxc-cli"]
    end

    MAIN --> MESSAGE
    MAIN --> IDENTITY
    MAIN --> TRANSPORT
    MESSAGE --> TRANSPORT
    MESSAGE --> NS
    IDENTITY --> NS
    TRANSPORT --> NS
    NS --> N44
    NS --> N59
    IDENTITY --> KP
    IDENTITY -.-> CLI_XC_EXT
```

## 14. Planned architecture ⬜ (EPICs 2-6)

```mermaid
graph LR
    subgraph EPIC2["EPIC 2 — Groupes"]
        GRP_KEY["NIP-44 + clé de groupe X25519"]
        CELL["Cellules de 3 (gift-wrap broadcast)"]
        INVITE["Invitation / join"]
    end

    subgraph EPIC3["EPIC 3 — Reticulum"]
        RNP["Transport Reticulum (RNP)"]
        SWITCH["Bascule auto Nostr ↔ Reticulum"]
    end

    subgraph EPIC5["EPIC 5 — Forward Secrecy"]
        DR["Double Ratchet"]
        ZERO_MEM["Zeroize mémoire"]
    end

    subgraph EPIC6["EPIC 6 — Relais"]
        RPI["Raspberry Pi 5 + Docker"]
        IPFS["Cache + IPFS"]
        WAN["Configuration WAN"]
    end

    EPIC2 --> CORE["rr-core (future)"]
    EPIC3 --> CORE
    EPIC5 --> CORE
    EPIC6 --> RELAY_NODE["rr-relay ⬜"]
```
