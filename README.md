# `limine-rs`

![workflow](https://github.com/limine-bootloader/limine-rs/workflows/Build/badge.svg)
![crates.io](https://img.shields.io/crates/d/limine)
![crates.io](https://img.shields.io/crates/v/limine)
![docs.rs](https://docs.rs/limine/badge.svg)

Rust crate for parsing the limine boot protocol structures.

## Resources
* [Limine Boot Protocol Specification](https://github.com/limine-bootloader/limine/blob/trunk/PROTOCOL.md)

## Barebones
The project provides an example kernel which can be found in the `barebones` directory; to show you
how to set up a simple 64-bit **long mode**, **higher half** rust kernel using Limine. The
kernel is shipped with a build script (`barebones/build.sh`) which is used to build the
rust kernel, create the ISO file and run the kernel in QEMU.

**Note**: In order to compile and run the barebones kernel, **nightly** rust is required.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, 
as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
