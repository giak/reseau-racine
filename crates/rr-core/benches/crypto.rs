use criterion::{
    async_executor::FuturesExecutor, criterion_group, criterion_main, BenchmarkId, Criterion,
    Throughput,
};
use nostr::nips::nip44;
use nostr::nips::nip59;
use nostr::{EventBuilder, Keys, Kind, Tag};
use std::hint::black_box;

fn bench_nip44_encrypt(c: &mut Criterion) {
    let (alice, bob) = (Keys::generate(), Keys::generate());
    let sk = alice.secret_key();
    let pk = bob.public_key();
    let mut group = c.benchmark_group("nip44_encrypt");
    for size in [64u64, 1024, 65408] {
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
    for size in [64u64, 1024, 65408] {
        let content = "A".repeat(size as usize);
        let ciphertext = nip44::encrypt(
            alice.secret_key(),
            &bob.public_key(),
            &content,
            nip44::Version::V2,
        )
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
        b.to_async(FuturesExecutor).iter(|| async {
            EventBuilder::new(Kind::GiftWrap, "benchmark payload")
                .sign(black_box(&keys))
                .await
                .unwrap()
        })
    });
}

fn bench_giftwrap_roundtrip(c: &mut Criterion) {
    let (alice, bob) = (Keys::generate(), Keys::generate());
    let alice_pk = alice.public_key();
    let bob_pk = bob.public_key();
    let mut group = c.benchmark_group("giftwrap_roundtrip");
    for size in [64u64, 1024, 32000] {
        let content = "A".repeat(size as usize);
        group.throughput(Throughput::Bytes(size));
        group.bench_with_input(BenchmarkId::from_parameter(size), &content, |b, content| {
            let content = content.clone();
            let b_alice = alice.clone();
            let b_bob = bob.clone();
            b.to_async(FuturesExecutor).iter(move || {
                let content = content.clone();
                let alice = b_alice.clone();
                let bob = b_bob.clone();
                async move {
                    let rumor =
                        EventBuilder::new(Kind::PrivateDirectMessage, &content).build(alice_pk);
                    let gift_wrap =
                        EventBuilder::gift_wrap(&alice, &bob_pk, rumor, Vec::<Tag>::new())
                            .await
                            .unwrap();
                    nip59::extract_rumor(&bob, &gift_wrap).await.unwrap();
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
