//! Performance benchmarks for vudo-credit

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::sync::Arc;
use tokio::runtime::Runtime;
use vudo_credit::{
    BftCommittee, CreditAccountHandle, DeviceEscrow, MutualCreditScheduler, TransactionMetadata,
};
use vudo_state::StateEngine;

/// Benchmark local spend operation (target: < 1ms)
fn bench_local_spend(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let (scheduler, _account) = rt.block_on(async {
        let state_engine = Arc::new(StateEngine::new().await.unwrap());
        let bft_committee = Arc::new(
            BftCommittee::new(vec![
                "m1".to_string(),
                "m2".to_string(),
                "m3".to_string(),
                "m4".to_string(),
            ])
            .unwrap(),
        );

        let scheduler = MutualCreditScheduler::new(
            Arc::clone(&state_engine),
            Arc::clone(&bft_committee),
            "device1".to_string(),
        )
        .await
        .unwrap();

        let account = CreditAccountHandle::create(&state_engine, "alice".to_string(), 1_000_000)
            .await
            .unwrap();

        let escrow = DeviceEscrow::new("device1".to_string(), 500_000, 7);
        scheduler.escrow_manager.set("alice", "device1", escrow);

        (scheduler, account)
    });

    c.bench_function("local_spend", |b| {
        b.to_async(&rt).iter(|| async {
            scheduler
                .spend_local(
                    black_box("alice"),
                    black_box(100),
                    black_box("bob"),
                    TransactionMetadata {
                        description: "Benchmark payment".to_string(),
                        category: None,
                        invoice_id: None,
                    },
                )
                .await
                .unwrap();
        });
    });
}

/// Benchmark escrow allocation
fn bench_escrow_allocation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let (scheduler, account) = rt.block_on(async {
        let state_engine = Arc::new(StateEngine::new().await.unwrap());
        let bft_committee = Arc::new(
            BftCommittee::new(vec![
                "m1".to_string(),
                "m2".to_string(),
                "m3".to_string(),
                "m4".to_string(),
            ])
            .unwrap(),
        );

        let scheduler = MutualCreditScheduler::new(
            Arc::clone(&state_engine),
            Arc::clone(&bft_committee),
            "device1".to_string(),
        )
        .await
        .unwrap();

        let account = CreditAccountHandle::create(&state_engine, "alice".to_string(), 1_000_000)
            .await
            .unwrap();

        (scheduler, account)
    });

    c.bench_function("escrow_allocation", |b| {
        b.to_async(&rt).iter(|| async {
            scheduler
                .request_escrow_refresh(black_box("alice"))
                .await
                .unwrap();
        });
    });
}

/// Benchmark overdraft detection
fn bench_overdraft_detection(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("overdraft_detection");

    for num_transactions in [10, 50, 100, 500] {
        group.throughput(Throughput::Elements(num_transactions as u64));

        let (scheduler, account) = rt.block_on(async {
            let state_engine = Arc::new(StateEngine::new().await.unwrap());
            let bft_committee = Arc::new(
                BftCommittee::new(vec![
                    "m1".to_string(),
                    "m2".to_string(),
                    "m3".to_string(),
                    "m4".to_string(),
                ])
                .unwrap(),
            );

            let scheduler = MutualCreditScheduler::new(
                Arc::clone(&state_engine),
                Arc::clone(&bft_committee),
                "device1".to_string(),
            )
            .await
            .unwrap();

            let account =
                CreditAccountHandle::create(&state_engine, "alice".to_string(), 10_000)
                    .await
                    .unwrap();

            // Add transactions
            account
                .update(|acc| {
                    for i in 0..num_transactions {
                        acc.add_transaction(vudo_credit::Transaction::new(
                            "alice".to_string(),
                            "bob".to_string(),
                            100,
                            TransactionMetadata::default(),
                        ));
                    }
                    Ok(())
                })
                .unwrap();

            (scheduler, account)
        });

        group.bench_with_input(
            BenchmarkId::from_parameter(num_transactions),
            &num_transactions,
            |b, _| {
                b.to_async(&rt).iter(|| async {
                    scheduler
                        .detect_overdrafts(black_box("alice"))
                        .await
                        .unwrap();
                });
            },
        );
    }

    group.finish();
}

/// Benchmark BFT reconciliation
fn bench_bft_reconciliation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("bft_reconciliation");

    for committee_size in [4, 7, 10, 13] {
        group.bench_with_input(
            BenchmarkId::from_parameter(committee_size),
            &committee_size,
            |b, &size| {
                let (scheduler, account) = rt.block_on(async {
                    let state_engine = Arc::new(StateEngine::new().await.unwrap());
                    let members: Vec<String> = (0..size).map(|i| format!("m{}", i)).collect();
                    let bft_committee = Arc::new(BftCommittee::new(members).unwrap());

                    let scheduler = MutualCreditScheduler::new(
                        Arc::clone(&state_engine),
                        Arc::clone(&bft_committee),
                        "device1".to_string(),
                    )
                    .await
                    .unwrap();

                    let account =
                        CreditAccountHandle::create(&state_engine, "alice".to_string(), 10_000)
                            .await
                            .unwrap();

                    account
                        .update(|acc| {
                            acc.add_transaction(vudo_credit::Transaction::new(
                                "alice".to_string(),
                                "bob".to_string(),
                                1_000,
                                TransactionMetadata::default(),
                            ));
                            Ok(())
                        })
                        .unwrap();

                    (scheduler, account)
                });

                b.to_async(&rt).iter(|| async {
                    scheduler
                        .reconcile_account(black_box("alice"))
                        .await
                        .unwrap();
                });
            },
        );
    }

    group.finish();
}

/// Benchmark escrow manager operations
fn bench_escrow_manager(c: &mut Criterion) {
    use vudo_credit::EscrowManager;

    let manager = EscrowManager::new();
    let escrow = DeviceEscrow::new("device1".to_string(), 10_000, 7);

    // Seed with some escrows
    for i in 0..100 {
        manager.set(&format!("account{}", i), "device1", escrow.clone());
    }

    c.bench_function("escrow_get", |b| {
        b.iter(|| {
            manager
                .get(black_box("account50"), black_box("device1"))
                .unwrap()
        });
    });

    c.bench_function("escrow_spend", |b| {
        b.iter(|| {
            manager.spend(black_box("account50"), black_box("device1"), black_box(10))
        });
    });

    c.bench_function("escrow_total_allocated", |b| {
        b.iter(|| manager.total_allocated(black_box("account50")));
    });
}

criterion_group!(
    benches,
    bench_local_spend,
    bench_escrow_allocation,
    bench_overdraft_detection,
    bench_bft_reconciliation,
    bench_escrow_manager
);

criterion_main!(benches);
