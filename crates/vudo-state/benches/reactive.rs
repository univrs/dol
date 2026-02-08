//! Benchmarks for reactive subscriptions.

use automerge::ROOT;
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use vudo_state::document_store::*;
use vudo_state::reactive::*;

fn benchmark_subscription_create(c: &mut Criterion) {
    let observable = ChangeObservable::new();
    let mut counter = 0;

    c.bench_function("subscription_create", |b| {
        b.iter(|| {
            counter += 1;
            let doc_id = DocumentId::new("users", &format!("user{}", counter));
            let filter = SubscriptionFilter::Document(doc_id);
            black_box(observable.subscribe(filter));
        });
    });
}

fn benchmark_change_notify(c: &mut Criterion) {
    let observable = ChangeObservable::new();
    let doc_id = DocumentId::new("users", "alice");

    c.bench_function("change_notify", |b| {
        b.iter(|| {
            let event = ChangeEvent {
                document_id: doc_id.clone(),
                timestamp: 0,
                change_hash: vec![],
                path: None,
            };
            observable.notify(black_box(event));
        });
    });
}

fn benchmark_change_notify_with_subscribers(c: &mut Criterion) {
    let mut group = c.benchmark_group("notify_with_subscribers");

    for num_subs in [1, 10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_subs),
            num_subs,
            |b, &num_subs| {
                let observable = ChangeObservable::new();
                let doc_id = DocumentId::new("users", "alice");
                let filter = SubscriptionFilter::Document(doc_id.clone());

                // Create subscriptions
                for _ in 0..num_subs {
                    observable.subscribe(filter.clone());
                }

                b.iter(|| {
                    let event = ChangeEvent {
                        document_id: doc_id.clone(),
                        timestamp: 0,
                        change_hash: vec![],
                        path: None,
                    };
                    observable.notify(event);
                    observable.flush_batch();
                });
            },
        );
    }

    group.finish();
}

fn benchmark_reactive_update(c: &mut Criterion) {
    let store = DocumentStore::new();
    let observable = ChangeObservable::new();
    let doc_id = DocumentId::new("users", "alice");
    let handle = store.create(doc_id.clone()).unwrap();

    // Create some subscriptions
    let filter = SubscriptionFilter::Document(doc_id);
    for _ in 0..10 {
        observable.subscribe(filter.clone());
    }

    let mut counter = 0i64;

    c.bench_function("reactive_update", |b| {
        b.iter(|| {
            counter += 1;
            handle
                .update_reactive(&observable, |doc| {
                    doc.put(ROOT, "counter", black_box(counter))?;
                    Ok(())
                })
                .unwrap();
        });
    });
}

fn benchmark_subscription_filtering(c: &mut Criterion) {
    let observable = ChangeObservable::new();

    // Create subscriptions with different filters
    for i in 0..100 {
        let doc_id = DocumentId::new("users", &format!("user{}", i));
        let filter = SubscriptionFilter::Document(doc_id);
        observable.subscribe(filter);
    }

    c.bench_function("subscription_filtering", |b| {
        b.iter(|| {
            let doc_id = DocumentId::new("users", "user50");
            let event = ChangeEvent {
                document_id: doc_id,
                timestamp: 0,
                change_hash: vec![],
                path: None,
            };
            observable.notify(event);
            observable.flush_batch();
        });
    });
}

fn benchmark_path_subscription(c: &mut Criterion) {
    let observable = ChangeObservable::new();
    let doc_id = DocumentId::new("users", "alice");
    let filter = SubscriptionFilter::Path(doc_id.clone(), "profile/*/name".to_string());
    observable.subscribe(filter);

    c.bench_function("path_subscription_notify", |b| {
        b.iter(|| {
            let event = ChangeEvent {
                document_id: doc_id.clone(),
                timestamp: 0,
                change_hash: vec![],
                path: Some("profile/public/name".to_string()),
            };
            observable.notify(black_box(event));
            observable.flush_batch();
        });
    });
}

criterion_group!(
    benches,
    benchmark_subscription_create,
    benchmark_change_notify,
    benchmark_change_notify_with_subscribers,
    benchmark_reactive_update,
    benchmark_subscription_filtering,
    benchmark_path_subscription,
);

criterion_main!(benches);
