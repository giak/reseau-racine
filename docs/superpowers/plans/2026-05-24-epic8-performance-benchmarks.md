# EPIC 8 — Performance Benchmarks Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add criterion benchmarks for crypto (4 benches, 10 metrics) and transport (4 benches), a `rr bench` CLI wrapper, and CI with baseline regression detection.

**Architecture:** Two criterion bench files (`benches/crypto.rs` via FuturesExecutor, `benches/transport.rs` via tokio Runtime), CLI wrapper in main.rs that calls `cargo bench` and parses output, CI job with baseline cache + grep-based regression check.

**Tech Stack:** criterion v0.8, tokio runtime, nostr crate (nip44, nip59, EventBuilder), nostr-sdk (transport), GitHub Actions cache.

---

### Task 1: Add criterion dev-dependency + [[bench]] sections

**Files:**
- Modify: `crates/rr-core/Cargo.toml`

- [ ] **Step 1: Update Cargo.toml**

```toml
[dev-dependencies]
proptest.workspace = true
serial_test.workspace = true
criterion = { version = "0.8", features = ["html_reports", "async_futures"] }

[[bench]]
name = "crypto"
harness = false

[[bench]]
name = "transport"
harness = false
```

- [ ] **Step 2: Verify Cargo.toml**

Run: `./scripts/dev.sh cargo metadata --format-version 1 > /dev/null`
Expected: no errors

- [ ] **Step 3: Commit**

```bash
rtk git add crates/rr-core/Cargo.toml
rtk git commit -m "build: add criterion + bench targets (crypto, transport)"
```

---

### Task 2: Create crypto benchmarks

**Files:**
- Create: `crates/rr-core/benches/crypto.rs`

- [ ] **Step 1: Write benches/crypto.rs**

```rust
use criterion::{
    async_executor::FuturesExecutor, black_box, criterion_group, criterion_main, BenchmarkId,
    Criterion, Throughput,
};
use nostr::nips::nip44;
use nostr::nips::nip59;
use nostr::{EventBuilder, Keys, Kind};

fn bench_nip44_encrypt(c: &mut Criterion) {
    let (alice, bob) = (Keys::generate(), Keys::generate());
    let sk = alice.secret_key();
    let pk = bob.public_key();
    let mut group = c.benchmark_group("nip44_encrypt");
    for size in [64u64, 1024, 65535] {
        let content = "A".repeat(size as usize);
        group.throughput(Throughput::Bytes(size));
        group.bench_with_input(BenchmarkId::from_parameter(size), &content, |b, content| {
            let content = content.clone();
            b.to_async(FuturesExecutor).iter(move || {
                let content = content.clone();
                async move {
                    nip44::encrypt(black_box(sk), black_box(&pk), &content, nip44::Version::V2)
                        .unwrap()
                }
            })
        });
    }
    group.finish();
}

fn bench_nip44_decrypt(c: &mut Criterion) {
    let (alice, bob) = (Keys::generate(), Keys::generate());
    let sk = bob.secret_key();
    let pk = alice.public_key();
    let mut group = c.benchmark_group("nip44_decrypt");
    for size in [64u64, 1024, 65535] {
        let content = "A".repeat(size as usize);
        let ciphertext =
            nip44::encrypt(alice.secret_key(), &bob.public_key(), &content, nip44::Version::V2)
                .unwrap();
        group.throughput(Throughput::Bytes(size));
        group.bench_with_input(BenchmarkId::from_parameter(size), &ciphertext, |b, ct| {
            let ct = ct.clone();
            b.to_async(FuturesExecutor).iter(move || {
                let ct = ct.clone();
                async move { nip44::decrypt(black_box(sk), black_box(&pk), &ct).unwrap() }
            })
        });
    }
    group.finish();
}

fn bench_event_sign(c: &mut Criterion) {
    let keys = Keys::generate();
    c.bench_function("event_sign", |b| {
        b.to_async(FuturesExecutor)
            .iter(|| async { EventBuilder::new(Kind::GiftWrap, "benchmark payload", &[]).sign(black_box(&keys)).await.unwrap() })
    });
}

fn bench_giftwrap_roundtrip(c: &mut Criterion) {
    let (alice, bob) = (Keys::generate(), Keys::generate());
    let bob_pk = bob.public_key();
    let mut group = c.benchmark_group("giftwrap_roundtrip");
    for size in [64u64, 1024, 65535] {
        let content = "A".repeat(size as usize);
        group.throughput(Throughput::Bytes(size));
        group.bench_with_input(BenchmarkId::from_parameter(size), &content, |b, content| {
            let content = content.clone();
            b.to_async(FuturesExecutor).iter(move || {
                let content = content.clone();
                async move {
                    let rumor = EventBuilder::new(Kind::PrivateDirectMessage, &content, &[])
                        .to_unsigned_event();
                    let gift_wrap = EventBuilder::gift_wrap(&alice, &bob_pk, rumor, &[])
                        .await
                        .unwrap();
                    nip59::extract_rumor(&bob, &gift_wrap).await.unwrap()
                }
            })
        });
    }
    group.finish();
}

criterion_group! {
    name = crypto;
    config = Criterion::default().configure_from_args();
    targets = bench_nip44_encrypt, bench_nip44_decrypt, bench_event_sign, bench_giftwrap_roundtrip
}
criterion_main!(crypto);
```

- [ ] **Step 2: Build and run crypto benchmarks**

Run: `./scripts/dev.sh cargo bench --bench crypto`
Expected: 10 metrics (3×3 sizes + event_sign), all PASS, Throughput column visible

- [ ] **Step 3: Commit**

```bash
rtk git add crates/rr-core/benches/crypto.rs
rtk git commit -m "bench: crypto — nip44 encrypt/decrypt, event_sign, giftwrap (3 sizes each)"
```

---

### Task 3: Create transport benchmarks

**Files:**
- Create: `crates/rr-core/benches/transport.rs`

- [ ] **Step 1: Write benches/transport.rs**

```rust
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, SamplingMode};
use nostr_sdk::prelude::*;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;

fn relay_url() -> String {
    std::env::var("RR_RELAY").unwrap_or_else(|_| "ws://172.20.0.2:8080".to_string())
}

fn setup(rt: &Runtime) -> (Arc<Client>, Keys) {
    let keys = Keys::generate();
    let client = Arc::new(rt.block_on(async { Client::new(&keys) }));
    let url = relay_url();
    rt.block_on(async {
        client.add_relay(&url).unwrap();
        client.connect().await;
        client.wait_for_connection(Some(Duration::from_secs(10))).await;
    });
    (client, keys)
}

fn bench_publish_single(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let (client, _sender) = setup(&rt);
    let receiver = Keys::generate().public_key();

    c.bench_function("publish_single", |b| {
        b.to_async(&rt).iter(|| async {
            client
                .send_private_msg(black_box(receiver), "benchmark payload", vec![])
                .await
                .unwrap();
        })
    });
}

fn bench_publish_batch(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let (client, _sender) = setup(&rt);
    let receiver = Keys::generate().public_key();
    let mut group = c.benchmark_group("publish_batch");
    group.sampling_mode(SamplingMode::Auto);

    for n in [1u64, 10, 100] {
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.to_async(&rt).iter(|| async {
                for _ in 0..n {
                    client
                        .send_private_msg(black_box(receiver), "benchmark payload", vec![])
                        .await
                        .unwrap();
                }
            })
        });
    }
    group.finish();
}

fn bench_sync_single(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let (client, _sender) = setup(&rt);
    let receiver = Keys::generate().public_key();

    rt.block_on(async {
        client
            .send_private_msg(receiver, "sync test", vec![])
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_millis(300)).await;
    });

    c.bench_function("sync_single", |b| {
        b.to_async(&rt).iter(|| async {
            let filter = Filter::new().kind(Kind::GiftWrap).pubkey(receiver).limit(1);
            client.subscribe(filter, None).await.unwrap();
            let mut notifications = client.notifications();
            while let Some(notification) = notifications.next().await {
                if let RelayPoolNotification::Event { event, .. } = notification {
                    if event.kind == Kind::GiftWrap {
                        break;
                    }
                }
            }
        })
    });
}

fn bench_sync_load(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let (client, _sender) = setup(&rt);
    let receiver = Keys::generate().public_key();
    let mut group = c.benchmark_group("sync_load");
    group.sampling_mode(SamplingMode::Auto);

    for n in [1u64, 10, 100] {
        rt.block_on(async {
            for _ in 0..n {
                client
                    .send_private_msg(receiver, "sync load test", vec![])
                    .await
                    .unwrap();
            }
            tokio::time::sleep(Duration::from_millis(300)).await;
        });

        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.to_async(&rt).iter(|| async {
                let filter = Filter::new().kind(Kind::GiftWrap).pubkey(receiver).limit(n);
                client.subscribe(filter, None).await.unwrap();
                let mut received = 0u64;
                let mut notifications = client.notifications();
                while received < n {
                    if let RelayPoolNotification::Event { event, .. } =
                        notifications.next().await
                    {
                        if event.kind == Kind::GiftWrap {
                            received += 1;
                        }
                    }
                }
            })
        });
    }
    group.finish();
}

criterion_group! {
    name = transport;
    config = Criterion::default().configure_from_args();
    targets = bench_publish_single, bench_publish_batch, bench_sync_single, bench_sync_load
}
criterion_main!(transport);
```

- [ ] **Step 2: Build transport bench**

Run: `./scripts/dev.sh cargo bench --bench transport -- --quick`
Expected: builds + runs against nostr-relay Docker (auto-started by dev.sh)

- [ ] **Step 3: Commit**

```bash
rtk git add crates/rr-core/benches/transport.rs
rtk git commit -m "bench: transport — publish/sync single + batch (requires nostr-relay)"
```

---

### Task 4: Add `rr bench` CLI command

**Files:**
- Modify: `crates/rr-cli/src/main.rs`

- [ ] **Step 1: Add Bench variant to Commands enum**

After `Restore`, add:

```rust
    /// Exécuter les benchmarks de performance
    Bench {
        #[arg(long)]
        crypto_only: bool,
        #[arg(long)]
        transport_only: bool,
        #[arg(long, default_value = "ws://172.20.0.2:8080")]
        relay: String,
    },
```

- [ ] **Step 2: Add bench handler in the match block**

After `Commands::Restore { phrase } => cmd_restore(phrase).await,`, add:

```rust
        Commands::Bench { crypto_only, transport_only, relay } => {
            cmd_bench(*crypto_only, *transport_only, relay).await
        }
```

- [ ] **Step 3: Add `cmd_bench` function and `check_relay` helper**

Add before `async fn cmd_init(...)`:

```rust
fn check_relay(url: &str) -> bool {
    let host = url
        .trim_start_matches("ws://")
        .trim_start_matches("wss://");
    let addr = host.parse::<std::net::SocketAddr>().unwrap_or(
        std::net::SocketAddr::new(
            std::net::IpAddr::V4(std::net::Ipv4Addr::new(172, 20, 0, 2)),
            8080,
        ),
    );
    std::net::TcpStream::connect_timeout(&addr, std::time::Duration::from_secs(2)).is_ok()
}

async fn cmd_bench(crypto_only: bool, transport_only: bool, relay: &str) {
    let run_crypto = !transport_only;
    let run_transport = !crypto_only;

    if run_crypto {
        println!("→ Running crypto benchmarks...");
        let status = std::process::Command::new("cargo")
            .args(["bench", "--bench", "crypto", "--", "--output-format", "bencher"])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::inherit())
            .status();

        match status {
            Ok(s) if s.success() => println!("  ✅ Crypto benchmarks done"),
            Ok(s) => eprintln!("  ⚠️  Crypto benchmarks exited with code: {}", s),
            Err(e) => eprintln!("  ❌ Failed to run cargo bench: {}", e),
        }
    }

    if run_transport {
        println!("→ Checking relay at {}...", relay);
        if !check_relay(relay) {
            println!("  ⚠️  Relay {} unreachable, skipping transport benchmarks", relay);
            return;
        }

        println!("→ Running transport benchmarks...");
        let status = std::process::Command::new("cargo")
            .args(["bench", "--bench", "transport", "--", "--output-format", "bencher"])
            .env("RR_RELAY", relay)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::inherit())
            .status();

        match status {
            Ok(s) if s.success() => println!("  ✅ Transport benchmarks done"),
            Ok(s) => eprintln!("  ⚠️  Transport benchmarks exited with code: {}", s),
            Err(e) => eprintln!("  ❌ Failed to run cargo bench: {}", e),
        }
    }
}
```

- [ ] **Step 4: Build CLI**

Run: `./scripts/dev.sh cargo build --package rr-cli`
Expected: compiles without errors

Run: `./scripts/dev.sh cargo run --package rr-cli -- bench --help`
Expected: shows bench subcommand flags

- [ ] **Step 5: Commit**

```bash
rtk git add crates/rr-cli/src/main.rs
rtk git commit -m "feat(cli): add rr bench — wraps cargo bench --bench crypto/transport"
```

---

### Task 5: Add bench job to CI workflow

**Files:**
- Modify: `.github/workflows/ci.yml`

- [ ] **Step 1: Add bench job**

After the `build-cli:` job (before `udeps:`), add:

```yaml
  bench:
    name: bench
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: swatinem/rust-cache@v2
      - name: Restore baseline cache
        uses: actions/cache@v4
        with:
          path: target/criterion
          key: bench-crypto-${{ github.sha }}
          restore-keys: bench-crypto-
      - name: Run benchmarks
        run: cargo bench --bench crypto -- --quick 2>&1 | tee bench-output.txt
      - name: Check for regression
        run: |
          if grep -q "Performance has regressed" bench-output.txt; then
            echo "❌ Performance regression >5% detected!"
            grep -B2 "regressed" bench-output.txt
            exit 1
          fi
          echo "✅ No significant performance changes"
```

- [ ] **Step 2: Optionally add bench to release job needs**

At `release:` job's `needs:` list (line ~172), add `bench`:

```yaml
needs: [lint, test, audit, fuzz, udeps, build-cli, bench]
```

- [ ] **Step 3: Verify YAML**

Run: `./scripts/dev.sh bash -c 'python3 -c "import yaml; yaml.safe_load(open(\"/workspace/.github/workflows/ci.yml\"))" && echo "YAML OK"'`
Expected: "YAML OK"

- [ ] **Step 4: Commit**

```bash
rtk git add .github/workflows/ci.yml
rtk git commit -m "ci: add bench job — crypto benchmarks with regression check"
```

---

### Task 6: End-to-end verification

**Files:** (none)

- [ ] **Step 1: Run crypto bench**

Run: `./scripts/dev.sh cargo bench --bench crypto`
Expected: 10 metrics, all PASS, Throughput visible

- [ ] **Step 2: Run full test suite**

Run: `./scripts/dev.sh cargo test --workspace --exclude rr-tauri --locked`
Expected: all existing tests pass (31 unit + 3 integ + 3 proptest)

- [ ] **Step 3: Run clippy**

Run: `./scripts/dev.sh cargo clippy --workspace --exclude rr-tauri -- -D warnings`
Expected: 0 warnings

- [ ] **Step 4: Run fmt**

Run: `./scripts/dev.sh cargo fmt --all --check`
Expected: no formatting changes

- [ ] **Step 5: Final commit**

```bash
rtk git diff --stat
rtk git add -A
rtk git commit -m "chore: final cleanup and verify"
```
