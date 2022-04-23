# poolio

[![crates.io][crates-badge]][crates-url]
[![GPL licensed][license-badge]][license-url]
[![CI][actions-badge]][actions-url]

[crates-badge]: https://img.shields.io/crates/v/poolio.svg
[crates-url]: https://crates.io/crates/poolio
[license-badge]: https://img.shields.io/badge/license-GPL-blue.svg
[license-url]: ./Cargo.toml
[actions-badge]: https://github.com/aronpaulson/poolio/actions/workflows/ci.yaml/badge.svg
[actions-url]: https://github.com/aronpaulson/poolio/actions/workflows/ci.yaml

A **simple** and **safe** thread-pool.

- simple API
- thread-safety:
  * rust's ownership and borrowing preventing data races
  * pure message-passing concurrency without deadlocks

For documumentation see [Released API docs](https://docs.rs/poolio).
In particular, you can find a design- and usage-description there.

## Contributing

If you want to contribute: [CONTRIBUTING](CONTRIBUTING.md).

### Security

For security-related issues see: [SECURITY](SECURITY.md).
