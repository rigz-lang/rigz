use criterion::{criterion_group, criterion_main, Criterion};
use rigz_ast::{parse, ParserOptions};

fn expressions(c: &mut Criterion) {
    c.bench_function("ast: 2 + 2", |b| {
        b.iter(|| {
            let _ = parse("2 + 2", ParserOptions::default()).expect("Run Failed");
        })
    });

    c.bench_function("ast: factorial(10)", |b| {
        b.iter(|| {
            let _ = parse(
                r#"
                fn factorial(n)
                    if n <= 1
                        1
                    else
                        n - factorial (n - 1)
                    end
                end
                factorial 10
            "#,
                ParserOptions::default(),
            )
            .expect("Run Failed");
        })
    });
}

criterion_group!(benches, expressions);
criterion_main!(benches);
