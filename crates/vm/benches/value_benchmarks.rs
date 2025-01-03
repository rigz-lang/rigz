use criterion::{criterion_group, criterion_main, Criterion};
use rigz_vm::Value;
use std::hint::black_box;

fn value_benchmark(c: &mut Criterion) {
    c.bench_function("Value: 2 + 2", |b| {
        b.iter(|| {
            let lhs: Value = black_box(2).into();
            let rhs: Value = black_box(2).into();
            &lhs + &rhs
        })
    });

    c.bench_function("Value: 2 * 2", |b| {
        b.iter(|| {
            let lhs: Value = black_box(2).into();
            let rhs: Value = black_box(2).into();
            &lhs * &rhs
        })
    });

    c.bench_function("Value: 2 / 2", |b| {
        b.iter(|| {
            let lhs: Value = black_box(2).into();
            let rhs: Value = black_box(2).into();
            &lhs / &rhs
        })
    });
}

criterion_group!(benches, value_benchmark);
criterion_main!(benches);
