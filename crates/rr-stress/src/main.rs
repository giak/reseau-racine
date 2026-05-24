use clap::Parser;
use nostr::key::SecretKey;
use nostr::{Keys, Tag};
use nostr_sdk::prelude::*;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use nostr::hashes::sha256;
use nostr::hashes::Hash as NostrHash;

#[derive(Parser)]
#[command(name = "rr-stress", about = "Load simulation for nostr-relay")]
struct Args {
    #[arg(long, default_value = "ws://172.20.0.2:8080")]
    relay: String,
    #[arg(long, default_value_t = 10)]
    users: usize,
    #[arg(long, default_value_t = 10)]
    messages: usize,
    #[arg(long = "interval", default_value_t = 100)]
    interval_ms: u64,
    #[arg(long, default_value = "true")]
    pre_connect: bool,
    #[arg(long, default_value_t = 4)]
    parallelism: usize,
    #[arg(long)]
    output: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize)]
struct Metrics {
    users: usize,
    total_messages: usize,
    success: u64,
    failed: u64,
    latency_ms: LatencyStats,
    errors: ErrorBreakdown,
    duration_sec: f64,
    throughput_msg_s: f64,
}

#[derive(Debug, Clone, Serialize)]
struct LatencyStats {
    p50: f64,
    p95: f64,
    p99: f64,
}

#[derive(Debug, Clone, Serialize, Default)]
struct ErrorBreakdown {
    timeout: u64,
    reject: u64,
    disconnect: u64,
}

struct SharedState {
    success_count: AtomicU64,
    fail_count: AtomicU64,
    latencies: Mutex<Vec<f64>>,
    errors: Mutex<ErrorBreakdown>,
}

fn keys_from_index(index: usize) -> Keys {
    let preimage = format!("rr-stress-seed-{:05}", index);
    let hash = sha256::Hash::hash(preimage.as_bytes());
    SecretKey::from_slice(hash.as_ref())
        .map(Keys::new)
        .expect("valid secret key")
}

async fn create_clients(relay: &str, users: usize, pre_connect: bool) -> Vec<Client> {
    let mut clients = Vec::with_capacity(users);
    for i in 0..users {
        let keys = keys_from_index(i);
        let client = Client::new(keys);
        client.add_relay(relay).await.unwrap();
        if pre_connect {
            client.connect().await;
            client.wait_for_connection(Duration::from_secs(10)).await;
        }
        clients.push(client);
    }
    clients
}

async fn run_phase(clients: &[Client], msgs: usize, interval: Duration, parallelism: usize, state: &Arc<SharedState>) {
    let n = clients.len();
    if n == 0 {
        return;
    }
    let semaphore = Arc::new(tokio::sync::Semaphore::new(parallelism));
    let mut handles = Vec::with_capacity(n);

    for (i, client) in clients.iter().enumerate() {
        let client = client.clone();
        let state = Arc::clone(state);
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        handles.push(tokio::spawn(async move {
            drop(permit);
            for _ in 0..msgs {
                let target = (i + 1) % n;
                let receiver = keys_from_index(target).public_key();
                let start = Instant::now();
                match client
                    .send_private_msg(receiver, "stress test payload", Vec::<Tag>::new())
                    .await
                {
                    Ok(_) => {
                        state.success_count.fetch_add(1, Ordering::SeqCst);
                        state
                            .latencies
                            .lock()
                            .unwrap()
                            .push(start.elapsed().as_secs_f64() * 1000.0);
                    }
                    Err(e) => {
                        state.fail_count.fetch_add(1, Ordering::SeqCst);
                        let mut errors = state.errors.lock().unwrap();
                        let msg = e.to_string();
                        if msg.contains("timeout") {
                            errors.timeout += 1;
                        } else if msg.contains("reject") || msg.contains("blocked") {
                            errors.reject += 1;
                        } else {
                            errors.disconnect += 1;
                        }
                    }
                }
                tokio::time::sleep(interval).await;
            }
        }));
    }

    for h in handles {
        h.await.unwrap();
    }
}

async fn run_stress(args: Args) {
    let state = Arc::new(SharedState {
        success_count: AtomicU64::new(0),
        fail_count: AtomicU64::new(0),
        latencies: Mutex::new(Vec::new()),
        errors: Mutex::new(ErrorBreakdown::default()),
    });

    let start = Instant::now();
    let interval = Duration::from_millis(args.interval_ms);

    println!("→ Creating {} clients...", args.users);
    let clients = create_clients(&args.relay, args.users, args.pre_connect).await;
    println!("  ✅ {} clients connected", clients.len());

    println!("→ Phase 1: Hello (1 msg each)...");
    run_phase(&clients, 1, interval, args.parallelism, &state).await;
    println!("  ✅ Hello done");

    if args.messages > 1 {
        let remaining = args.messages - 1;
        println!("→ Phase 2: Chat ({} msgs each)...", remaining);
        run_phase(&clients, remaining, interval, args.parallelism, &state).await;
        println!("  ✅ Chat done");
    }

    let elapsed = start.elapsed().as_secs_f64();
    let total =
        state.success_count.load(Ordering::SeqCst) + state.fail_count.load(Ordering::SeqCst);
    let mut latencies = state.latencies.lock().unwrap().clone();
    latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let errors = state.errors.lock().unwrap();

    let metrics = Metrics {
        users: args.users,
        total_messages: total as usize,
        success: state.success_count.load(Ordering::SeqCst),
        failed: state.fail_count.load(Ordering::SeqCst),
        latency_ms: LatencyStats {
            p50: percentile(&latencies, 50.0),
            p95: percentile(&latencies, 95.0),
            p99: percentile(&latencies, 99.0),
        },
        errors: ErrorBreakdown {
            timeout: errors.timeout,
            reject: errors.reject,
            disconnect: errors.disconnect,
        },
        duration_sec: elapsed,
        throughput_msg_s: if elapsed > 0.0 {
            total as f64 / elapsed
        } else {
            0.0
        },
    };

    let json = serde_json::to_string_pretty(&metrics).unwrap();
    if let Some(path) = &args.output {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        std::fs::write(path, &json).unwrap();
        println!("✅ Results saved to {:?}", path);
    }
    println!("{}", json);
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((p / 100.0) * (sorted.len() as f64 - 1.0)).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    println!(
        "→ rr-stress: {} users, {} msgs each, relay={}",
        args.users, args.messages, args.relay
    );
    run_stress(args).await;
}
