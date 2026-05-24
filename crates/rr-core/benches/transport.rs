use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, SamplingMode};
use nostr_sdk::prelude::*;
use std::hint::black_box;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;

fn relay_url() -> String {
    std::env::var("RR_RELAY").unwrap_or_else(|_| "ws://172.20.0.2:8080".to_string())
}

fn connect_client(rt: &Runtime, keys: Keys) -> Arc<Client> {
    let client = Arc::new(rt.block_on(async { Client::new(keys) }));
    let url = relay_url();
    rt.block_on(async {
        client.add_relay(&url).await.unwrap();
        client.connect().await;
        client.wait_for_connection(Duration::from_secs(10)).await;
    });
    client
}

fn setup_pair(rt: &Runtime) -> (Arc<Client>, Arc<Client>, PublicKey) {
    let sender = connect_client(rt, Keys::generate());
    let receiver_keys = Keys::generate();
    let receiver_pk = receiver_keys.public_key();
    let receiver = connect_client(rt, receiver_keys);
    (sender, receiver, receiver_pk)
}

fn bench_publish_single(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let (client, _receiver, _receiver_pk) = setup_pair(&rt);
    let sender_pk = Keys::generate().public_key();

    c.bench_function("publish_single", |b| {
        b.iter(|| {
            rt.block_on(async {
                client
                    .send_private_msg(black_box(sender_pk), "benchmark payload", vec![])
                    .await
                    .unwrap();
            })
        })
    });
}

fn bench_publish_batch(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let (client, _receiver, _receiver_pk) = setup_pair(&rt);
    let sender_pk = Keys::generate().public_key();
    let mut group = c.benchmark_group("publish_batch");
    group.sampling_mode(SamplingMode::Auto);

    for n in [1u64, 10, 100] {
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| {
                rt.block_on(async {
                    for _ in 0..n {
                        client
                            .send_private_msg(black_box(sender_pk), "benchmark payload", vec![])
                            .await
                            .unwrap();
                    }
                })
            })
        });
    }
    group.finish();
}

fn bench_sync_single(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let (sender, receiver, receiver_pk) = setup_pair(&rt);

    rt.block_on(async {
        sender
            .send_private_msg(receiver_pk, "sync test", vec![])
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_millis(300)).await;
    });

    c.bench_function("sync_single", |b| {
        b.iter(|| {
            rt.block_on(async {
                let filter = Filter::new()
                    .kind(Kind::GiftWrap)
                    .pubkey(receiver_pk)
                    .limit(1usize);
                let events = receiver
                    .fetch_events(filter, Duration::from_secs(10))
                    .await
                    .unwrap();
                assert!(!events.is_empty());
            })
        })
    });
}

fn bench_sync_load(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let (sender, receiver, receiver_pk) = setup_pair(&rt);
    let mut group = c.benchmark_group("sync_load");
    group.sampling_mode(SamplingMode::Auto);

    for n in [1u64, 10, 100] {
        rt.block_on(async {
            for _ in 0..n {
                sender
                    .send_private_msg(receiver_pk, "sync load test", vec![])
                    .await
                    .unwrap();
            }
            tokio::time::sleep(Duration::from_millis(300)).await;
        });

        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| {
                rt.block_on(async {
                    let filter = Filter::new()
                        .kind(Kind::GiftWrap)
                        .pubkey(receiver_pk)
                        .limit(n as usize);
                    let events = receiver
                        .fetch_events(filter, Duration::from_secs(10))
                        .await
                        .unwrap();
                    assert_eq!(events.len() as u64, n);
                })
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
