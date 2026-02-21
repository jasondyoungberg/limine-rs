// Copyright © 2026, __robot@PLT
// SPDX-License-Identifier: MIT

#![no_std]
#![feature(ptr_metadata)]
#![allow(unused)]

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
    magic: [u64; 3],
}

impl BaseRevision {
    /// Use the default base revision (4).
    pub const fn new() -> Self {
        Self::with_revision(4)
    }

    /// Use a specific base revision.
    pub const fn with_revision(revision: u64) -> Self {
        Self {
            magic: [0xf9562b2d5c95a6c8, 0x6a7b384944536bdc, revision],
        }
    }

    /// Whether the requested revision is supported.
    pub const fn is_supported(&self) -> bool {
        self.magic[2] == 0
    }

    /// What revision is actually in use right now, regardless of whether it is the requested one.
    pub const fn actual_revision(&self) -> Option<u64> {
        if self.magic[1] == 0x6a7b384944536bdc {
            None
        } else {
            Some(self.magic[1])
        }
    }
}
