use clap::Parser;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex};

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

#[tokio::main]
async fn main() {
    let args = Args::parse();
    println!(
        "→ rr-stress: {} users, {} msgs each, relay={}",
        args.users, args.messages, args.relay
    );
    run_stress(args).await;
}

async fn run_stress(_args: Args) {
    let _state = Arc::new(SharedState {
        success_count: AtomicU64::new(0),
        fail_count: AtomicU64::new(0),
        latencies: Mutex::new(Vec::new()),
        errors: Mutex::new(ErrorBreakdown::default()),
    });
    todo!("Tasks 3-6 will fill this")
}
