mod bencher_cores;
use bencher_cores::execute;

mod macros;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    let sizes = vec![1, 2, 4, 6, 8, 12];

    for size in sizes {
        c.bench_function(
            bench_identifier!("execute", "poolio", size),
            bencher!(execute, poolio::ThreadPool, size),
        );

        c.bench_function(
            bench_identifier!("execute", "rayon", size),
            bencher!(execute, rayon::ThreadPool, size),
        );
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
