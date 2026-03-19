// Copyright © 2026, Julian Scheffers
// SPDX-License-Identifier: MIT OR Apache-2.0

#![no_std]
#![allow(unused)]
#![feature(ptr_metadata)]
#![feature(unsafe_cell_access)]

use core::cell::UnsafeCell;

use request::BootloaderInfoRequest;

pub mod entrypoint;
pub mod file;
pub mod firmware;
pub mod framebuffer;
pub mod memmap;
pub mod module;
pub mod mp;
pub mod paging;
pub mod request;
pub mod uuid;

/// Common magic for requests.
pub const COMMON_MAGIC: [u64; 2] = [0xc7b1dd30df4c8b88, 0x0a82e883a194f07b];

/// Request that Limine uses a certain base revision.
/// If omitted, 0 is used.
#[repr(C)]
pub struct BaseRevision {
    magic: UnsafeCell<[u64; 3]>,
}
unsafe impl Send for BaseRevision {}
unsafe impl Sync for BaseRevision {}

impl BaseRevision {
    /// Use the default base revision (5).
    pub const fn new() -> Self {
        Self::with_revision(5)
    }

    /// Use a specific base revision.
    pub const fn with_revision(revision: u64) -> Self {
        Self {
            magic: UnsafeCell::new([0xf9562b2d5c95a6c8, 0x6a7b384944536bdc, revision]),
        }
    }

    /// Whether the requested revision is supported.
    pub fn is_supported(&self) -> bool {
        unsafe { (self.magic.get() as *const u64).add(2).read_volatile() == 0 }
    }

    /// What revision is actually in use right now, regardless of whether it is the requested one.
    pub fn actual_revision(&self) -> Option<u64> {
        let actual = unsafe { (self.magic.get() as *const u64).add(1).read_volatile() };
        if actual == 0x6a7b384944536bdc {
            None
        } else {
            Some(actual)
        }
    }
}
