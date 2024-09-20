#![no_std]
#![deny(missing_docs)]

//! Rust Bindings for the limine boot protocol.
//!
//! # Examples
//! An example kernel using this crate can be found
//! [here](https://github.com/limine-bootloader/limine-rust-barebones). For
//! smaller usage examples, see [usage](#usage).
//!
//! # Crate features
//! - `uuid`: Implements `Into<uuid::Uuid>` and `From<uuid::Uuid>` for [`file::Uuid`].
//! - `ipaddr`: Enables functions in [`file::File`] to return `Ipv4Addr`. This
//!   is feature gated because it will only appear in stable on Rust 1.77.0, on
//!   March 21st.
//!
//! # Revisions
//! Many types in the limine boot protocol have associated revisions. These
//! signify various added fields and changes to the protocol. For requests, if a
//! bootloader doesn't support a revision, it will simply process it as if it is
//! the highest revision it does support. The bootloader will only return the
//! latest supported revision in responses. The response revision is
//! automatically checked by the types in this crate, and types that may not be
//! returned are wrapped in an [`Option`].
//!
//! In this crate, you may specify the revisions of your requests via
//! `with_revision`, but you likely just want to use `new`. This will use the
//! latest revision supported by this crate.
//!
//! # Usage
//! The first thing you need to do is place a [`BaseRevision`] tag somewhere in
//! your code. This tag is used to identify what revision of the protocol your
//! kernel requires. Without this tag, the bootloader will assume revision 0,
//! which is likely not what you want.
//!
//! The [`BaseRevision`] tag can be placed anywhere in your code, like so:
//! ```rust
//! use limine::BaseRevision;
//!
//! // Require version 2 or higher
//! pub static BASE_REVISION: BaseRevision = BaseRevision::new();
//! ```
//!
//! Next, you can place any requests you would like. For example, to request a
//! larger stack (*recommended on debug Rust builds*), you can do the following:
//! ```rust
//! use limine::request::StackSizeRequest;
//!
//! // Some reasonable size
//! pub const STACK_SIZE: u64 = 0x100000;
//!
//! // Request a larger stack
//! pub static STACK_SIZE_REQUEST: StackSizeRequest = StackSizeRequest::new().with_size(STACK_SIZE);
//! ```

// `Default` is not const anyway, so implementing is not very useful.
#![allow(clippy::new_without_default)]

use core::cell::UnsafeCell;

pub mod file;
pub mod firmware_type;
pub mod framebuffer;
pub mod memory_map;
pub mod modules;
pub mod paging;
pub mod request;
pub mod response;
pub mod smp;

/// A tag setting the base revision supported by the kernel. Set this in your
/// kernel in order to require a higher revision. Without this tag, the
/// bootloader will assume revision 0.
///
/// The latest revision is 2.
#[repr(C)]
pub struct BaseRevision {
    _id: [u64; 2],
    revision: UnsafeCell<u64>,
}
impl BaseRevision {
    /// Create a new base revision tag with the latest revision.
    pub const fn new() -> Self {
        Self::with_revision(2)
    }

    /// Create a new base revision tag with the given revision.
    pub const fn with_revision(revision: u64) -> Self {
        Self {
            _id: [0xf9562b2d5c95a6c8, 0x6a7b384944536bdc],
            revision: UnsafeCell::new(revision),
        }
    }

    /// Check whether the given revision is supported by the bootloader.
    pub fn is_supported(&self) -> bool {
        (unsafe { self.revision.get().read_volatile() }) == 0
    }
}
unsafe impl Sync for BaseRevision {}
unsafe impl Send for BaseRevision {}

#[cfg(not(any(
    target_arch = "x86_64",
    target_arch = "aarch64",
    target_arch = "riscv64",
    target_arch = "loongarch64"
)))]
compile_error!(
    "Unsupported architecture, please use `x86_64`, `aarch64`, `riscv64`, or `loongarch64`"
);
