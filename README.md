# `limine-rs`

[![build](https://github.com/limine-bootloader/limine-rs/workflows/Build/badge.svg)](https://github.com/limine-bootloader/limine-rs/actions)
![downloads](https://img.shields.io/crates/d/limine)
[![version](https://img.shields.io/crates/v/limine)](https://crates.io/crates/limine)
[![docs](https://docs.rs/limine/badge.svg)](https://docs.rs/limine)
![license](https://img.shields.io/crates/l/limine)

Rust crate for parsing the limine boot protocol structures.

## Resources
* [Limine Boot Protocol Specification](https://github.com/limine-bootloader/limine/blob/trunk/PROTOCOL.md)

## Barebones
The project provides an example kernel which can be found in the `barebones` directory; to show you
how to set up a simple 64-bit **long mode**, **higher half** rust kernel using Limine.

The kernel ships with a shell script (`.cargo/runner.sh`) which can be invoked by executing
`cargo run` in the barebones directory. This script will package the built kernel executable into
an ISO image with the bootloader and run it with QEMU.

**Note**: In order to compile and run the barebones kernel, **nightly** rust is required.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you,
as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
