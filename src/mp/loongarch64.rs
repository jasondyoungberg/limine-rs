// Copyright © 2026, Julian Scheffers
// SPDX-License-Identifier: MIT OR Apache-2.0

use core::sync::atomic::{AtomicPtr, AtomicU64};

/// Signature for the entrypoint on processor hand-over from the bootloader.
pub type MpGotoFunction = fn(&MpInfo) -> !;

/// Information about a single processor.
#[repr(C)]
pub struct MpInfo {
    _resvd0: u64,
}

#[repr(C)]
pub struct MpRespData {
    cpu_count: u64,
    cpus: *const (),
}
