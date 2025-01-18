//! Auxiliary types for the [MP request](crate::request::MpRequest).

use core::sync::atomic::{AtomicPtr, Ordering};

use bitflags::bitflags;

/// A function pointer that the core will jump to when it is written to.
#[repr(transparent)]
pub struct GotoAddress {
    inner: AtomicPtr<()>,
}
impl GotoAddress {
    /// Set the goto address pointer. This will cause the core to jump to the
    /// given function. This function also synchronizes all writes, so anything
    /// written before this function returns is guaranteed to be seen before the
    /// core jumps to the given function.
    pub fn write(&self, func: unsafe extern "C" fn(&Cpu) -> !) {
        self.inner.store(func as *mut (), Ordering::SeqCst);
    }
}

/// A CPU entry in the MP request.
#[repr(C)]
#[cfg(target_arch = "x86_64")]
pub struct Cpu {
    /// The ACPI processor ID, according to the ACPI MADT.
    pub id: u32,

    /// The APIC ID, according to the ACPI MADT.
    pub lapic_id: u32,

    _reserved: core::mem::MaybeUninit<u64>,

    /// The address to jump to. Writing to this field will cause the core to
    /// jump to the given function. The function will receive a pointer to this
    /// structure, and it will have its own 64KiB (or requested-size) stack.
    pub goto_address: GotoAddress,

    /// Free for use by the executable.
    pub extra: u64,
}

/// A CPU entry in the MP request.
#[repr(C)]
#[cfg(target_arch = "aarch64")]
pub struct Cpu {
    /// The ACPI processor ID, according to the ACPI MADT.
    pub id: u32,

    _reserved1: core::mem::MaybeUninit<u32>,

    /// The MPIDR of the CPU, according to the ACPI MADT or the device tree.
    pub mpidr: u64,

    _reserved: core::mem::MaybeUninit<u64>,

    /// The address to jump to. Writing to this field will cause the core to
    /// jump to the given function. The function will receive a pointer to this
    /// structure, and it will have its own 64KiB (or requested-size) stack.
    pub goto_address: GotoAddress,

    /// Free for use by the executable.
    pub extra: u64,
}

/// A CPU entry in the MP request.
#[repr(C)]
#[cfg(target_arch = "riscv64")]
pub struct Cpu {
    /// The ACPI processor ID, according to the ACPI MADT.
    pub id: u64,

    /// The hart ID, according to the ACPI MADT or the device tree.
    pub hartid: u64,

    _reserved: core::mem::MaybeUninit<u64>,

    /// The address to jump to. Writing to this field will cause the core to
    /// jump to the given function. The function will receive a pointer to this
    /// structure, and it will have its own 64KiB (or requested-size) stack.
    pub goto_address: GotoAddress,

    /// Free for use by the executable.
    pub extra: u64,
}

/// A CPU entry in the MP request.
#[repr(C)]
#[cfg(target_arch = "loongarch64")]
pub struct Cpu {
    _reserved: core::mem::MaybeUninit<u64>,
}

bitflags! {
    /// Flags for the [MP request](crate::request::MpRequest).
    #[derive(Default, Clone, Copy)]
    pub struct RequestFlags: u64 {
        /// Initialize the X2APIC.
        #[cfg(target_arch = "x86_64")]
        const X2APIC = 1 << 0;
    }
}

#[cfg(target_arch = "x86_64")]
bitflags! {
    /// Flags for the [MP response](crate::response::MpResponse).
    #[derive(Default, Clone, Copy)]
    pub struct ResponseFlags: u32 {
        /// The X2APIC was initialized.
        #[cfg(target_arch = "x86_64")]
        const X2APIC = 1 << 0;
    }
}

#[cfg(not(target_arch = "x86_64"))]
bitflags! {
    /// Flags for the [MP response](crate::response::MpResponse).
    #[derive(Default, Clone, Copy)]
    pub struct ResponseFlags: u64 {}
}
