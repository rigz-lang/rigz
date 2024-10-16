use criterion::{criterion_group, criterion_main, Criterion};
use rigz_vm::VMBuilder;

fn builder_benchmark(c: &mut Criterion) {
    c.bench_function("Builder: 2 + 2", |b| {
        b.iter(|| {
            let _ = VMBuilder::new()
                .add_load_instruction(2, 2.into())
                .add_load_instruction(3, 2.into())
                .add_add_instruction(2, 3, 4)
                .build();
        })
    });
}

fn vm_benchmark(c: &mut Criterion) {
    c.bench_function("VM(build): 2 + 2", |b| {
        b.iter(|| {
            VMBuilder::new()
                .add_load_instruction(2, 2.into())
                .add_load_instruction(3, 2.into())
                .add_add_instruction(2, 3, 4)
                .build()
                .run()
                .expect("Failed to run");
        })
    });
}

criterion_group!(benches, builder_benchmark, vm_benchmark);
criterion_main!(benches);