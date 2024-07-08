//! Auxiliary types for the [paging mode
//! request](crate::request::PagingModeRequest).

use bitflags::bitflags;

bitflags! {
    /// Paging mode flags. None are currently specified.
    #[derive(Debug, Default, Clone, Copy)]
    pub struct Flags: u64 {}
}

/// A paging mode.
#[repr(transparent)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Mode(u64);
impl From<u64> for Mode {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
impl Mode {
    /// (x86_64 and aarch64) Four-level paging (i.e. 48-bit virtual addresses on x86_64).
    pub const FOUR_LEVEL: Self = Self(0);
    /// (x86_64 and aarch64) Five-level paging (i.e. 52-bit virtual addresses on x86_64).
    pub const FIVE_LEVEL: Self = Self(1);
}

#[cfg(target_arch = "riscv64")]
impl Mode {
    /// (riscv64 only) SV39, i.e. 39-bit virtual addresses.
    pub const SV39: Self = Self(0);
    /// (riscv64 only) SV48, i.e. 48-bit virtual addresses.
    pub const SV48: Self = Self(1);
    /// (riscv64 only) SV57, i.e. 57-bit virtual addresses.
    pub const SV57: Self = Self(2);
}
