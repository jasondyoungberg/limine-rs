//! Auxiliary types for the [memory map request](crate::request::MemoryMapRequest)

use core::fmt::Debug;

/// A type of entry within the memory map.
#[repr(transparent)]
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct EntryType(u64);
impl EntryType {
    /// The memory region is freely usable.
    pub const USABLE: Self = Self(0);
    /// The memory region is permanently reserved.
    pub const RESERVED: Self = Self(1);
    /// The memory region is currently used by ACPI, but can be reclaimed once
    /// ACPI structures are no longer needed.
    pub const ACPI_RECLAIMABLE: Self = Self(2);
    /// The memory region is permanently reserved by ACPI, and must not be used.
    pub const ACPI_NVS: Self = Self(3);
    /// The memory region is unusable due to physical damage or similar errors.
    pub const BAD_MEMORY: Self = Self(4);
    /// The memory region is used by the bootloader, but can be reclaimed once
    /// all responses have been processed and will no longer be used.
    pub const BOOTLOADER_RECLAIMABLE: Self = Self(5);
    /// The memory region is used by the kernel and modules, and as such is
    /// permanently reserved.
    pub const KERNEL_AND_MODULES: Self = Self(6);
    /// The memory region is used by the framebuffer, and as such is permanently
    /// reserved.
    pub const FRAMEBUFFER: Self = Self(7);
}
impl From<u64> for EntryType {
    fn from(val: u64) -> Self {
        Self(val)
    }
}
impl Debug for EntryType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            &Self::USABLE => write!(f, "EntryType::USABLE"),
            &Self::RESERVED => write!(f, "EntryType::RESERVED"),
            &Self::ACPI_RECLAIMABLE => write!(f, "EntryType::ACPI_RECLAIMABLE"),
            &Self::ACPI_NVS => write!(f, "EntryType::ACPI_NVS"),
            &Self::BAD_MEMORY => write!(f, "EntryType::BAD_MEMORY"),
            &Self::BOOTLOADER_RECLAIMABLE => write!(f, "EntryType::BOOTLOADER_RECLAIMABLE"),
            &Self::KERNEL_AND_MODULES => write!(f, "EntryType::KERNEL_AND_MODULES"),
            &Self::FRAMEBUFFER => write!(f, "EntryType::FRAMEBUFFER"),
            _ => write!(f, "EntryType({})", self.0),
        }
    }
}

/// A memory map entry.
#[repr(C)]
pub struct Entry {
    /// The base of the memory region, in *physical space*.
    pub base: u64,
    /// The length of the memory region, in bytes.
    pub length: u64,
    /// The type of the memory region. See [`EntryType`] for specific values.
    pub entry_type: EntryType,
}
impl Debug for Entry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Entry")
            .field("base", &format_args!("{:#x}", self.base))
            .field("length", &self.length)
            .field("entry_type", &self.entry_type)
            .finish()
    }
}
