//! Performance benchmarks for vudo-identity
//!
//! Target performance:
//! - Peer DID creation: < 50ms
//! - UCAN delegation verification: < 10ms

use chrono::Utc;
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use vudo_identity::{Capability, Did, Ucan};
use x25519_dalek::{PublicKey, StaticSecret};

fn bench_did_creation(c: &mut Criterion) {
    c.bench_function("did_creation", |b| {
        b.iter(|| {
            let signing_key = SigningKey::generate(&mut OsRng);
            let encryption_secret = StaticSecret::random_from_rng(&mut OsRng);
            let encryption_public = PublicKey::from(&encryption_secret);

            black_box(Did::from_keys(signing_key.verifying_key(), &encryption_public).unwrap())
        })
    });
}

fn bench_did_parsing(c: &mut Criterion) {
    let signing_key = SigningKey::generate(&mut OsRng);
    let encryption_secret = StaticSecret::random_from_rng(&mut OsRng);
    let encryption_public = PublicKey::from(&encryption_secret);
    let did = Did::from_keys(signing_key.verifying_key(), &encryption_public).unwrap();
    let did_str = did.as_str().to_string();

    c.bench_function("did_parsing", |b| {
        b.iter(|| black_box(Did::parse(&did_str).unwrap()))
    });
}

fn bench_did_document_generation(c: &mut Criterion) {
    let signing_key = SigningKey::generate(&mut OsRng);
    let encryption_secret = StaticSecret::random_from_rng(&mut OsRng);
    let encryption_public = PublicKey::from(&encryption_secret);
    let did = Did::from_keys(signing_key.verifying_key(), &encryption_public).unwrap();

    c.bench_function("did_document_generation", |b| {
        b.iter(|| black_box(did.to_document()))
    });
}

fn bench_ucan_creation_and_signing(c: &mut Criterion) {
    let issuer_key = SigningKey::generate(&mut OsRng);
    let issuer_enc = PublicKey::from(&StaticSecret::random_from_rng(&mut OsRng));
    let issuer_did = Did::from_keys(issuer_key.verifying_key(), &issuer_enc).unwrap();

    let audience_key = SigningKey::generate(&mut OsRng);
    let audience_enc = PublicKey::from(&StaticSecret::random_from_rng(&mut OsRng));
    let audience_did = Did::from_keys(audience_key.verifying_key(), &audience_enc).unwrap();

    c.bench_function("ucan_creation_and_signing", |b| {
        b.iter(|| {
            let ucan = Ucan::new(
                issuer_did.clone(),
                audience_did.clone(),
                vec![Capability::new("vudo://myapp/data", "read")],
                Utc::now().timestamp() as u64 + 3600,
                None,
                None,
                vec![],
            );

            black_box(ucan.sign(&issuer_key).unwrap())
        })
    });
}

fn bench_ucan_verification(c: &mut Criterion) {
    let issuer_key = SigningKey::generate(&mut OsRng);
    let issuer_enc = PublicKey::from(&StaticSecret::random_from_rng(&mut OsRng));
    let issuer_did = Did::from_keys(issuer_key.verifying_key(), &issuer_enc).unwrap();

    let audience_key = SigningKey::generate(&mut OsRng);
    let audience_enc = PublicKey::from(&StaticSecret::random_from_rng(&mut OsRng));
    let audience_did = Did::from_keys(audience_key.verifying_key(), &audience_enc).unwrap();

    let ucan = Ucan::new(
        issuer_did.clone(),
        audience_did.clone(),
        vec![Capability::new("vudo://myapp/data", "read")],
        Utc::now().timestamp() as u64 + 3600,
        None,
        None,
        vec![],
    )
    .sign(&issuer_key)
    .unwrap();

    c.bench_function("ucan_verification", |b| {
        b.iter(|| black_box(ucan.verify().unwrap()))
    });
}

fn bench_ucan_delegation_chain(c: &mut Criterion) {
    let mut group = c.benchmark_group("ucan_delegation_chain");

    // Create delegation chains of varying lengths
    for chain_length in [1, 3, 5, 10].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(chain_length),
            chain_length,
            |b, &length| {
                // Build delegation chain
                let mut keys = Vec::new();
                let mut dids = Vec::new();

                for _ in 0..=length {
                    let key = SigningKey::generate(&mut OsRng);
                    let enc = PublicKey::from(&StaticSecret::random_from_rng(&mut OsRng));
                    let did = Did::from_keys(key.verifying_key(), &enc).unwrap();
                    keys.push(key);
                    dids.push(did);
                }

                // Create root UCAN
                let mut ucan = Ucan::new(
                    dids[0].clone(),
                    dids[1].clone(),
                    vec![Capability::wildcard("vudo://myapp/")],
                    Utc::now().timestamp() as u64 + 3600,
                    None,
                    None,
                    vec![],
                )
                .sign(&keys[0])
                .unwrap();

                // Build delegation chain
                for i in 1..length {
                    ucan = ucan
                        .delegate(
                            dids[i + 1].clone(),
                            vec![Capability::new("vudo://myapp/data", "read")],
                            Utc::now().timestamp() as u64 + 1800,
                            &keys[i],
                        )
                        .unwrap();
                }

                // Benchmark verification
                b.iter(|| black_box(ucan.verify().unwrap()))
            },
        );
    }

    group.finish();
}

fn bench_ucan_encoding_decoding(c: &mut Criterion) {
    let issuer_key = SigningKey::generate(&mut OsRng);
    let issuer_enc = PublicKey::from(&StaticSecret::random_from_rng(&mut OsRng));
    let issuer_did = Did::from_keys(issuer_key.verifying_key(), &issuer_enc).unwrap();

    let audience_key = SigningKey::generate(&mut OsRng);
    let audience_enc = PublicKey::from(&StaticSecret::random_from_rng(&mut OsRng));
    let audience_did = Did::from_keys(audience_key.verifying_key(), &audience_enc).unwrap();

    let ucan = Ucan::new(
        issuer_did.clone(),
        audience_did.clone(),
        vec![Capability::new("vudo://myapp/data", "read")],
        Utc::now().timestamp() as u64 + 3600,
        None,
        None,
        vec![],
    )
    .sign(&issuer_key)
    .unwrap();

    c.bench_function("ucan_encoding", |b| {
        b.iter(|| black_box(ucan.encode().unwrap()))
    });

    let jwt = ucan.encode().unwrap();
    c.bench_function("ucan_decoding", |b| {
        b.iter(|| black_box(Ucan::decode(&jwt).unwrap()))
    });
}

fn bench_capability_matching(c: &mut Criterion) {
    let cap_wildcard = Capability::wildcard("vudo://myapp/");
    let cap_specific = Capability::new("vudo://myapp/data", "read");

    c.bench_function("capability_matching", |b| {
        b.iter(|| black_box(cap_wildcard.matches(&cap_specific)))
    });
}

criterion_group!(
    benches,
    bench_did_creation,
    bench_did_parsing,
    bench_did_document_generation,
    bench_ucan_creation_and_signing,
    bench_ucan_verification,
    bench_ucan_delegation_chain,
    bench_ucan_encoding_decoding,
    bench_capability_matching,
);
criterion_main!(benches);
