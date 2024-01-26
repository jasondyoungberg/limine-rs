#![no_std]
#![deny(missing_docs)]

//! Rust Bindings for the limine boot protocol.

use core::cell::UnsafeCell;

pub mod file;
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
/// The latest revision is 1.
pub struct BaseRevision {
    _id: [u64; 2],
    revision: UnsafeCell<u64>,
}
impl BaseRevision {
    /// Create a new base revision tag with the given revision.
    pub const fn new(revision: u64) -> Self {
        Self {
            _id: [0xf9562b2d5c95a6c8, 0x6a7b384944536bdc],
            revision: UnsafeCell::new(revision),
        }
    }

    /// Check whether the given revision is supported by the bootloader.
    pub fn is_supported(&self) -> bool {
        (unsafe { *self.revision.get() }) == 0
    }
}
unsafe impl Sync for BaseRevision {}
unsafe impl Send for BaseRevision {}
