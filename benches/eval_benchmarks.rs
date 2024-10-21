use criterion::{criterion_group, criterion_main, Criterion};
use rigz_runtime::runtime::eval;

fn expressions(c: &mut Criterion) {
    c.bench_function("2 + 2", |b| {
        b.iter(|| {
            let _ = eval("2 + 2").expect("Run Failed");
        })
    });
}

criterion_group!(benches, expressions);
criterion_main!(benches);
