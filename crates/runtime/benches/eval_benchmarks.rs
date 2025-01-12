use criterion::{criterion_group, criterion_main, Criterion};
use rigz_runtime::runtime::eval;

fn expressions(c: &mut Criterion) {
    c.bench_function("2 + 2", |b| {
        b.iter(|| {
            let _ = eval("2 + 2".to_string()).expect("Run Failed");
        })
    });

    c.bench_function("factorial(10)", |b| {
        b.iter(|| {
            let _ = eval(
                r#"
                fn factorial(n)
                    if n <= 1
                        1
                    else
                        n - factorial (n - 1)
                    end
                end
                factorial 10
            "#
                .to_string(),
            )
            .expect("Run Failed");
        })
    });
}

criterion_group!(benches, expressions);
criterion_main!(benches);
