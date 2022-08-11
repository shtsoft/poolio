mod bencher_cores;
use bencher_cores::primes;

mod macros;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    let sizes = vec![1, 2, 4, 6, 8, 12];

    for size in sizes {
        c.bench_function(
            bench_identifier!("primes", "poolio", size),
            bencher!(primes, poolio::ThreadPool, size),
        );

        c.bench_function(
            bench_identifier!("primes", "rayon", size),
            bencher!(primes, rayon::ThreadPool, size),
        );
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
