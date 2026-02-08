//! Benchmarks for VUDO Privacy cryptographic operations.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use vudo_privacy::crypto::{PersonalDataCrypto, DataEncryptionKey};
use vudo_privacy::pseudonymous::PseudonymousActorId;

fn bench_dek_generation(c: &mut Criterion) {
    let crypto = PersonalDataCrypto::new();

    c.bench_function("dek_generation", |b| {
        b.iter(|| {
            crypto.generate_dek(black_box("did:peer:alice"))
        })
    });
}

fn bench_encryption(c: &mut Criterion) {
    let crypto = PersonalDataCrypto::new();
    let dek = crypto.generate_dek("did:peer:alice").unwrap();

    let mut group = c.benchmark_group("encryption");

    for size in [64, 256, 1024, 4096].iter() {
        let data = vec![0u8; *size];

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                crypto.encrypt_field(black_box(&dek), black_box(&data))
            })
        });
    }

    group.finish();
}

fn bench_decryption(c: &mut Criterion) {
    let crypto = PersonalDataCrypto::new();
    let dek = crypto.generate_dek("did:peer:alice").unwrap();

    let mut group = c.benchmark_group("decryption");

    for size in [64, 256, 1024, 4096].iter() {
        let data = vec![0u8; *size];
        let encrypted = crypto.encrypt_field(&dek, &data).unwrap();

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                crypto.decrypt_field(black_box(&dek), black_box(&encrypted))
            })
        });
    }

    group.finish();
}

fn bench_pseudonym_generation(c: &mut Criterion) {
    c.bench_function("pseudonym_generation", |b| {
        b.iter(|| {
            PseudonymousActorId::from_did(black_box("did:peer:alice"))
        })
    });
}

fn bench_encrypt_decrypt_roundtrip(c: &mut Criterion) {
    let crypto = PersonalDataCrypto::new();
    let dek = crypto.generate_dek("did:peer:alice").unwrap();
    let data = b"alice@example.com";

    c.bench_function("encrypt_decrypt_roundtrip", |b| {
        b.iter(|| {
            let encrypted = crypto.encrypt_field(black_box(&dek), black_box(data)).unwrap();
            crypto.decrypt_field(black_box(&dek), black_box(&encrypted)).unwrap()
        })
    });
}

fn bench_multiple_users(c: &mut Criterion) {
    let crypto = PersonalDataCrypto::new();

    c.bench_function("dek_generation_100_users", |b| {
        b.iter(|| {
            for i in 0..100 {
                let did = format!("did:peer:user_{}", i);
                crypto.generate_dek(black_box(&did)).unwrap();
            }
        })
    });
}

criterion_group!(
    benches,
    bench_dek_generation,
    bench_encryption,
    bench_decryption,
    bench_pseudonym_generation,
    bench_encrypt_decrypt_roundtrip,
    bench_multiple_users,
);

criterion_main!(benches);
