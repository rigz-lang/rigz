use criterion::{criterion_group, criterion_main, Criterion};
use rigz_core::Number;
use std::hint::black_box;

fn number_benchmark(c: &mut Criterion) {
    c.bench_function("Number: 2 + 2", |b| {
        b.iter(|| {
            let lhs: Number = black_box(2).into();
            let rhs: Number = black_box(2).into();
            &lhs + &rhs
        })
    });

    c.bench_function("Number: 2 * 2", |b| {
        b.iter(|| {
            let lhs: Number = black_box(2).into();
            let rhs: Number = black_box(2).into();
            &lhs * &rhs
        })
    });

    c.bench_function("Number: 2 / 2", |b| {
        b.iter(|| {
            let lhs: Number = black_box(2).into();
            let rhs: Number = black_box(2).into();
            &lhs / &rhs
        })
    });
}

criterion_group!(benches, number_benchmark);
criterion_main!(benches);
