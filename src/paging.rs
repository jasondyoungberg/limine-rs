//! Auxiliary types for the [paging mode
//! request](crate::request::PagingModeRequest).

/// A paging mode.
#[repr(transparent)]
#[derive(PartialEq, Eq, Clone, Copy)]
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

    /// The default paging mode.
    pub const DEFAULT: Self = Self::FOUR_LEVEL;
    /// The maximum supported paging mode.
    pub const MAX: Self = Self::FIVE_LEVEL;
    /// The minimum supported paging mode.
    pub const MIN: Self = Self::FOUR_LEVEL;
}

#[cfg(target_arch = "riscv64")]
impl Mode {
    /// (riscv64 only) SV39, i.e. 39-bit virtual addresses.
    pub const SV39: Self = Self(0);
    /// (riscv64 only) SV48, i.e. 48-bit virtual addresses.
    pub const SV48: Self = Self(1);
    /// (riscv64 only) SV57, i.e. 57-bit virtual addresses.
    pub const SV57: Self = Self(2);

    /// The default paging mode.
    pub const DEFAULT: Self = Self::SV48;
    /// The maximum supported paging mode.
    pub const MAX: Self = Self::SV57;
    /// The minimum supported paging mode.
    pub const MIN: Self = Self::SV39;
}

#[cfg(target_arch = "loongarch64")]
impl Mode {
    /// (loongarch64 only) Four-level paging.
    pub const FOUR_LEVEL: Self = Self(0);

    /// The default paging mode.
    pub const DEFAULT: Self = Self::FOUR_LEVEL;
    /// The maximum supported paging mode.
    pub const MAX: Self = Self::FOUR_LEVEL;
    /// The minimum supported paging mode.
    pub const MIN: Self = Self::FOUR_LEVEL;
}
