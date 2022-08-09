mod pool;

use pool::Pool;

fn foobar<P>(pool: P)
where
    P: Pool,
{
    const JOBS: usize = 100;

    for _ in 0..JOBS {
        let job = move || {};

        pool.execute(job);
    }
}

use criterion::black_box;

macro_rules! bench_identifier_foobar {
    ($cr8:literal, $size:expr) => {
        &format!("load-{}-pool-({}-worker(s))", $cr8, $size)
    };
}

macro_rules! bencher_foobar {
    ($Pool:ty, $size:expr) => {
        |b| {
            b.iter(|| {
                let pool: $Pool = Pool::new(black_box($size));
                foobar(pool);
            })
        }
    };
}

use criterion::{criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    let sizes = vec![1, 2, 4, 6, 8, 12];

    for size in sizes {
        c.bench_function(
            bench_identifier_foobar!("poolio", size),
            bencher_foobar!(poolio::ThreadPool, size),
        );

        c.bench_function(
            bench_identifier_foobar!("rayon", size),
            bencher_foobar!(rayon::ThreadPool, size),
        );
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
