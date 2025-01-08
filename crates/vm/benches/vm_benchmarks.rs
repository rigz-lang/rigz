use criterion::{criterion_group, criterion_main, Criterion};
use rigz_vm::{RigzBuilder, VMBuilder};

fn builder_benchmark(c: &mut Criterion) {
    c.bench_function("Builder: 2 + 2", |b| {
        b.iter(|| {
            let mut b = VMBuilder::new();
            b.add_load_instruction(2.into())
                .add_load_instruction(2.into())
                .add_add_instruction();
            let _ = b.build();
        })
    });
}

fn vm_benchmark(c: &mut Criterion) {
    c.bench_function("VM(skip build): 2 + 2", |b| {
        let mut builder = VMBuilder::new();
        builder
            .add_load_instruction(2.into())
            .add_load_instruction(2.into())
            .add_add_instruction();
        let mut vm = builder.build();
        b.iter(|| {
            vm.frames.current.get_mut().pc = 0;
            let _ = vm.eval().expect("Failed to run");
        })
    });
}

criterion_group!(benches, builder_benchmark, vm_benchmark);
criterion_main!(benches);
