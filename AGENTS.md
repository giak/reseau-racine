# Agent Commands

## Build
```bash
cargo build --workspace
cargo build --release
```

## Test
```bash
cargo test --workspace
cargo test --package rr-core
```

## Check
```bash
cargo check --workspace
cargo clippy --workspace
cargo fmt --all --check
```

## Run
```bash
cargo run --package rr-cli -- init
cargo run --package rr-cli -- identity
cargo run --package rr-cli -- help
```

## Debug
```bash
RUST_LOG=debug cargo run --package rr-cli -- <command>
```

## Quick check
```bash
cargo check --workspace && cargo test --package rr-core && cargo clippy --package rr-core --package rr-cli
```
