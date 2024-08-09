//! Auxiliary types for the [firmware type request
//! ](crate::request::FirmwareTypeRequest).

/// A firmware type.
#[repr(transparent)]
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct FirmwareType(u64);
impl FirmwareType {
    /// The firmware type is x86 BIOS.
    pub const X86_BIOS: Self = Self(0);
    /// The firmware type is 32-bit UEFI.
    pub const UEFI_32: Self = Self(1);
    /// The firmware type is 64-bit UEFI.
    pub const UEFI_64: Self = Self(2);
}
impl From<u64> for FirmwareType {
    fn from(value: u64) -> Self {
        Self(value)
    }
}
