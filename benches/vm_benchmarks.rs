use criterion::{criterion_group, criterion_main, Criterion};
use rigz_vm::{Binary, BinaryOperation, Clear, Instruction, VMBuilder};

fn builder_benchmark(c: &mut Criterion) {
    c.bench_function("Builder: 2 + 2", |b| {
        b.iter(|| {
            let mut b = VMBuilder::new();
            b.add_load_instruction(2, 2.into())
                .add_load_instruction(3, 2.into())
                .add_add_instruction(2, 3, 4);
            let _ = b.build();
        })
    });
}

fn vm_benchmark(c: &mut Criterion) {
    c.bench_function("VM(build): 2 + 2", |b| {
        b.iter(|| {
            let mut builder = VMBuilder::new();
            builder
                .add_load_instruction(2, 2.into())
                .add_load_instruction(3, 2.into())
                .add_add_instruction(2, 3, 4);
            builder.build().eval().expect("Failed to run");
        })
    });
}

fn vm_benchmark_clear(c: &mut Criterion) {
    c.bench_function("VM(clear): 2 + 2", |b| {
        b.iter(|| {
            let mut builder = VMBuilder::new();
            builder
                .add_load_instruction(2, 2.into())
                .add_load_instruction(3, 2.into())
                .add_instruction(Instruction::BinaryClear(
                    Binary {
                        lhs: 2,
                        rhs: 3,
                        output: 4,
                        op: BinaryOperation::Add,
                    },
                    Clear::Two(2, 3),
                ));
            builder.build().eval().expect("Failed to run");
        })
    });
}

criterion_group!(benches, builder_benchmark, vm_benchmark, vm_benchmark_clear);
criterion_main!(benches);
