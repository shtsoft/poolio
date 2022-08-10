mod pool;

use pool::Pool;

fn execute<P>(pool: P)
where
    P: Pool,
{
    const JOBS: usize = 100;

    for n in 0..JOBS {
        let compute_primes_less_than_n = move || {
            fn is_prime(i: usize) -> bool {
                for j in 2..(i / 2) {
                    if i % j == 0 {
                        return false;
                    } else {
                        continue;
                    }
                }
                true
            }

            let cap = n * n;

            let mut primes = vec![];

            for i in 2..cap {
                if is_prime(i) {
                    primes.push(i);
                };
            }

            let mut counter = 0;

            for _ in primes {
                counter += counter;
            }

            use std::io::*;

            let response = format!("There are {} primes which are less than {}.", counter, cap);

            sink().write_all(response.as_bytes()).unwrap();
        };

        pool.execute(compute_primes_less_than_n);
    }
}

use criterion::black_box;

macro_rules! bench_identifier_execute {
    ($cr8:literal, $size:expr) => {
        &format!("execute:{}-pool-({}-worker(s))", $cr8, $size)
    };
}

macro_rules! bencher_execute {
    ($Pool:ty, $size:expr) => {
        |b| {
            b.iter(|| {
                let pool: $Pool = Pool::new(black_box($size));
                execute(pool);
            })
        }
    };
}

use criterion::{criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    let sizes = vec![1, 2, 4, 6, 8, 12];

    for size in sizes {
        c.bench_function(
            bench_identifier_execute!("poolio", size),
            bencher_execute!(poolio::ThreadPool, size),
        );

        c.bench_function(
            bench_identifier_execute!("rayon", size),
            bencher_execute!(rayon::ThreadPool, size),
        );
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
