mod pool;

use pool::Pool;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn foobar(_: usize) {
    let _: poolio::ThreadPool = Pool::new(2);
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("foobar", |b| b.iter(|| foobar(black_box(0))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
