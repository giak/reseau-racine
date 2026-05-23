# Architecture Réseau Racine

## Vue d'ensemble

```mermaid
graph TB
    subgraph CLI["rr-cli (binary)"]
        INIT["rr init"]
        IDENTITY["rr identity"]
        CONTACTS["rr add-contact / contacts"]
        SEND["rr send"]
        SYNC["rr sync"]
        RESTORE["rr restore"]
        BENCH["rr bench"]
    end

    subgraph CORE["rr-core (library)"]
        CRYPTO["NIP-44 V2<br/>encrypt / decrypt"]
        ID["IdentityManager<br/>secp256k1 + BIP-39"]
        MSG["MessageService<br/>send / receive"]
        TRANS["NostrTransport<br/>connect / wait_for_connection"]
        KEYSTORE["KeySource<br/>file / keepassxc / keepass-rs"]
    end

    subgraph STRESS["rr-stress (binary)"]
        LOAD["Load simulation<br/>N users, N messages"]
        METRICS["Métriques<br/>p50/p95/p99/success"]
    end

    SEND --> MSG
    SYNC --> MSG
    MSG --> CRYPTO
    MSG --> TRANS
    SEND --> KEYSTORE
    IDENTITY --> ID
    CLI --> CORE
    STRESS --> CORE
    STRESS --> RELAY
    TRANS --> RELAY["nostr-relay<br/>Docker ws://172.20.0.2:8080"]
    BENCH --> CORE
```

## Flux message NIP-17 (EPIC 1)

```mermaid
sequenceDiagram
    participant A as Alice (rr send)
    participant RELAY as nostr-relay
    participant B as Bob (rr sync)

    Note over A: KeySource → nsec
    A->>A: NIP-44 encrypt(content, bob_pubkey)
    A->>A: Wrap in rumor (kind 14)
    A->>A: Seal with alice_privkey (kind 13)
    A->>A: GiftWrap with ephemeral key (kind 1059)
    A->>RELAY: publish event kind 1059
    Note over RELAY: Vérifie Output.success

    B->>RELAY: subscribe filter: kind=1059, #p=bob_pubkey
    RELAY-->>B: event kind 1059
    B->>B: Unwrap GiftWrap → seal → rumor
    B->>B: NIP-44 decrypt(content, alice_pubkey)
    Note over B: Affiche: 📨 alice: message
```

## Vault KeePassXC (EPIC 7)

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

## Benchmarks (EPIC 8)

```mermaid
flowchart LR
    subgraph Crypto_Bench["Bench Crypto (criterion)"]
        ENC["NIP-44 encrypt 1KB"]
        DEC["NIP-44 decrypt"]
        SIGN["Event sign kind 1059"]
        GW["GiftWrap full roundtrip<br/>encrypt→seal→unwrap→decrypt"]
    end

    subgraph Transport_Bench["Bench Transport (relais local)"]
        PUB1["Publish single<br/>connect→wait→publish"]
        PUBN["Publish batch<br/>N messages (1,10,100)"]
        SYNC1["Sync single<br/>subscribe→unwrap"]
        SYNCN["Sync load<br/>N messages→receive→unwrap"]
    end

    subgraph RUN["rr bench --count N"]
        CLI_RUN["rr bench --count 10"]
    end

    CLI_RUN --> Crypto_Bench
    CLI_RUN --> Transport_Bench

    PUB1 --> RELAY["nostr-relay Docker<br/>ws://172.20.0.2:8080"]
    PUBN --> RELAY
    SYNC1 --> RELAY
    SYNCN --> RELAY
```

## Simulation charge (EPIC 9)

```mermaid
flowchart TD
    SEED["Seed stable (index)"]
    GEN["Générer N identités<br/>déterministes"]
    CLIENTS["Créer N clients tokio<br/>Keys + Client nostr-sdk"]

    subgraph Phase_Hello["Phase Hello"]
        HELLO["Chaque user → 1 destinataire aléatoire<br/>Évite N×M broadcast"]
    end

    subgraph Phase_Chat["Phase Chat"]
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
    CLIENTS --> Phase_Hello
    CLIENTS --> Phase_Chat
    Phase_Hello --> COLLECT
    Phase_Chat --> COLLECT
    COLLECT --> OUTPUT
    OUTPUT --> RELAY["nostr-relay ws://172.20.0.2:8080"]
```

## Relations entre modules Rust

```mermaid
graph RL
    subgraph rr_cli["rr-cli"]
        MAIN["main.rs"]
    end

    subgraph rr_core["rr-core"]
        IDENTITY["identity.rs"]
        MESSAGE["message.rs"]
        TRANSPORT["transport/nostr.rs"]
    end

    subgraph external["Externe"]
        NS["nostr-sdk"]
        N44["nostr::nips::nip44"]
        N59["nostr::nips::nip59"]
    end

    subgraph keychain["KeePassXC"]
        KDBX["vault.kdbx"]
        CLI_XC["keepassxc-cli"]
    end

    MAIN --> MESSAGE
    MAIN --> IDENTITY
    MESSAGE --> TRANSPORT
    MESSAGE --> NS
    IDENTITY --> NS
    TRANSPORT --> NS
    NS --> N44
    NS --> N59
    IDENTITY --> KDBX
    IDENTITY -.-> CLI_XC
```

## Organisation fichiers

```
reseau-racine/
├── .devcontainer/        # Docker + services
├── .github/workflows/    # CI (13 jobs, 8 status checks)
├── crates/
│   ├── rr-cli/           # CLI binary (9 commandes)
│   ├── rr-core/          # Librairie (crypto, identity, message, transport)
│   │   └── fuzz/         # 3 fuzz targets (NIP-44 roundtrip/decrypt, identity)
│   └── rr-tauri/         # Tauri v2 (build bloqué: GTK)
├── docs/
│   ├── ARCHITECTURE.md   # Ce fichier
│   ├── TRACKING.md       # Suivi des EPICs
│   └── superpowers/
│       ├── specs/        # Specs approuvées
│       └── plans/        # Plans d'implémentation
└── scripts/
    └── dev.sh            # Wrapper Docker
```
