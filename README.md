# poolio

[![crates.io][crates-badge]][crates-url]
[![GPL licensed][license-badge]][license-url]
[![CI][actions-badge]][actions-url]

[crates-badge]: https://img.shields.io/crates/v/poolio.svg
[crates-url]: https://crates.io/crates/poolio
[license-badge]: https://img.shields.io/badge/license-GPL-blue.svg
[license-url]: ./Cargo.toml
[actions-badge]: https://github.com/shtsoft/poolio/actions/workflows/ci.yaml/badge.svg
[actions-url]: https://github.com/shtsoft/poolio/actions/workflows/ci.yaml

A **simple** and **safe** and **fast** thread pool based on pure message-passing concurrency defying the mainstream.

- simplicity:
  * small API
  * less than 300 lines of code
- safety:
  * no dependencies (apart from crossbeam)
  * thoroughly tested
  * memory-safety: no `unsafe`-code
  * thread-safety:
    + no data races
    + no deadlocks
- performance:
  * [crossbeam](https://github.com/crossbeam-rs/crossbeam)-channels
  * as fast as the most popular Rust [threadpool](https://github.com/rust-threadpool/rust-threadpool) (see [Benches](#benches))

For documumentation see [Released API docs](https://docs.rs/poolio).
In particular, you can find a design- and usage-description there.

### Benches

The [benches](benches) pit poolio against [threadpool](https://github.com/rust-threadpool/rust-threadpool) in a battle of computing various lists of primes and writing them to a sink.
On a computer 'Intel(R) Core(TM) i7-9750H CPU @ 2.60GHz' (6 cores and 12 CPUs) running x86\_64 GNU/Linux we measured the following average times for executing the job:

| Primes     | poolio     | threadpool |
| ---------- | ----------:| ----------:|
| 6 workers  | 27.468 ms  | 28.431 ms  |
| 12 workers | 24.056 ms  | 23.456 ms  |

This suggests that the poolio and threadpool are equally performant.
The full result can be downloaded [here](https://github.com/shtsoft/poolio/releases/latest/download/benches.tar.gz).
(The benchmarks are powered by [criterion](https://github.com/bheisler/criterion.rs).)

## Contributing

If you want to contribute: [CONTRIBUTING](CONTRIBUTING.md).

### Security

For security-related issues see: [SECURITY](SECURITY.md).
