use super::pool::Pool;

pub fn primes<P>(size: usize) -> Option<P>
where
    P: Pool,
{
    const JOBS: usize = 100;

    let pool: P = Pool::new(size);

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

    pool.join();

    None
}
