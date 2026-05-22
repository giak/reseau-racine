# Agent Commands

## Docker Dev Environment
All Rust commands run inside the dev container. Two ways:

```bash
# Option A — via dev.sh wrapper (recommended)
./scripts/dev.sh cargo check --workspace
./scripts/dev.sh cargo test --workspace

# Option B — direct docker compose
docker compose -f .devcontainer/compose.yaml exec -T dev cargo check --workspace

# Shell dans le container
./scripts/dev.sh
```

## Build
```bash
./scripts/dev.sh cargo build --workspace
./scripts/dev.sh cargo build --release --package rr-cli
```

## Test
```bash
./scripts/dev.sh cargo test --workspace --exclude rr-tauri
./scripts/dev.sh cargo test --package rr-core
```

## Check
```bash
./scripts/dev.sh cargo check --workspace --exclude rr-tauri
./scripts/dev.sh cargo clippy --workspace --exclude rr-tauri
./scripts/dev.sh cargo fmt --all --check
```

## Run
```bash
./scripts/dev.sh cargo run --package rr-cli -- init
./scripts/dev.sh cargo run --package rr-cli -- identity
./scripts/dev.sh cargo run --package rr-cli -- help
```

## Debug
```bash
./scripts/dev.sh env RUST_LOG=debug cargo run --package rr-cli -- <command>
```

## Services
```bash
# nostr-relay accessible via ws://nostr-relay:8080

# Logs du relais
docker compose -f .devcontainer/compose.yaml logs -f nostr-relay

# Redémarrer
docker compose -f .devcontainer/compose.yaml restart nostr-relay
```

## Quick check
```bash
./scripts/dev.sh sh -c "cargo check --workspace --exclude rr-tauri && cargo test --package rr-core && cargo clippy --package rr-core --package rr-cli"
```
