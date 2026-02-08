//! Benchmarks for document store operations.

use automerge::ROOT;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use vudo_state::document_store::*;

fn benchmark_document_store_create(c: &mut Criterion) {
    c.bench_function("document_store_create", |b| {
        let store = DocumentStore::new();
        let mut counter = 0;

        b.iter(|| {
            counter += 1;
            let doc_id = DocumentId::new("users", &format!("user{}", counter));
            black_box(store.create(doc_id).unwrap());
        });
    });
}

fn benchmark_document_store_get(c: &mut Criterion) {
    let store = DocumentStore::new();
    let doc_id = DocumentId::new("users", "alice");
    store.create(doc_id.clone()).unwrap();

    c.bench_function("document_store_get", |b| {
        b.iter(|| {
            black_box(store.get(&doc_id).unwrap());
        });
    });
}

fn benchmark_document_handle_update(c: &mut Criterion) {
    let store = DocumentStore::new();
    let doc_id = DocumentId::new("users", "alice");
    let handle = store.create(doc_id).unwrap();

    let mut counter = 0i64;

    c.bench_function("document_handle_update", |b| {
        b.iter(|| {
            counter += 1;
            handle
                .update(|doc| {
                    doc.put(ROOT, "counter", black_box(counter))?;
                    Ok(())
                })
                .unwrap();
        });
    });
}

fn benchmark_document_handle_read(c: &mut Criterion) {
    let store = DocumentStore::new();
    let doc_id = DocumentId::new("users", "alice");
    let handle = store.create(doc_id).unwrap();

    handle
        .update(|doc| {
            doc.put(ROOT, "name", "Alice")?;
            doc.put(ROOT, "age", 30i64)?;
            doc.put(ROOT, "email", "alice@example.com")?;
            Ok(())
        })
        .unwrap();

    c.bench_function("document_handle_read", |b| {
        b.iter(|| {
            handle
                .read(|doc| {
                    let _name: String = doc.get(ROOT, "name")?.unwrap().0.into();
                    let _age: i64 = doc.get(ROOT, "age")?.unwrap().0.into();
                    let _email: String = doc.get(ROOT, "email")?.unwrap().0.into();
                    Ok(())
                })
                .unwrap();
        });
    });
}

fn benchmark_document_save_load(c: &mut Criterion) {
    let store = DocumentStore::new();
    let doc_id = DocumentId::new("users", "alice");
    let handle = store.create(doc_id).unwrap();

    for i in 0..100 {
        handle
            .update(|doc| {
                doc.put(ROOT, format!("key{}", i), i as i64)?;
                Ok(())
            })
            .unwrap();
    }

    c.bench_function("document_save", |b| {
        b.iter(|| {
            black_box(handle.save());
        });
    });

    let bytes = handle.save();

    c.bench_function("document_load", |b| {
        b.iter(|| {
            let doc_id2 = DocumentId::new("users", "alice_copy");
            black_box(store.load(doc_id2, &bytes).unwrap());
        });
    });
}

criterion_group!(
    benches,
    benchmark_document_store_create,
    benchmark_document_store_get,
    benchmark_document_handle_update,
    benchmark_document_handle_read,
    benchmark_document_save_load,
);

criterion_main!(benches);
