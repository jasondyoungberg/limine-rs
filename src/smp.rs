//! Auxiliary types for the [SMP request](crate::request::SmpRequest).

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
        self.inner.store(func as *mut (), Ordering::Release);
    }
}

/// A CPU entry in the SMP request.
#[repr(C)]
pub struct Cpu {
    /// The ACPI processor ID, according to the ACPI MADT.
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    pub id: u32,
    /// The ACPI processor ID, according to the ACPI MADT.
    #[cfg(target_arch = "riscv64")]
    pub id: u64,

    /// The APIC ID, according to the ACPI MADT.
    #[cfg(target_arch = "x86_64")]
    pub lapic_id: u32,

    #[cfg(target_arch = "aarch64")]
    _reserved1: core::mem::MaybeUninit<u32>,
    /// The MPIDR of the CPU, according to the ACPI MADT or the device tree.
    #[cfg(target_arch = "aarch64")]
    pub mpidr: u64,

    /// The hart ID, according to the ACPI MADT or the device tree.
    #[cfg(target_arch = "riscv64")]
    pub hartid: u64,

    _reserved: core::mem::MaybeUninit<u64>,
    /// The address to jump to. Writing to this field will cause the core to
    /// jump to the given function. The function will receive a pointer to this
    /// structure, and it will have its own 64KiB (or requested-size) stack.
    #[cfg(not(target_arch = "loongarch64"))]
    pub goto_address: GotoAddress,
    /// Free for use by the kernel.
    #[cfg(not(target_arch = "loongarch64"))]
    pub extra: u64,
}

bitflags! {
    /// Flags for the [SMP request](crate::request::SmpRequest).
    #[derive(Default, Clone, Copy)]
    pub struct RequestFlags: u64 {
        /// Initialize the X2APIC.
        #[cfg(target_arch = "x86_64")]
        const X2APIC = 1 << 0;
    }
}

#[cfg(target_arch = "x86_64")]
bitflags! {
    /// Flags for the [SMP response](crate::response::SmpResponse).
    #[derive(Default, Clone, Copy)]
    pub struct ResponseFlags: u32 {
        /// The X2APIC was initialized.
        #[cfg(target_arch = "x86_64")]
        const X2APIC = 1 << 0;
    }
}

#[cfg(not(target_arch = "x86_64"))]
bitflags! {
    /// Flags for the [SMP response](crate::response::SmpResponse).
    #[derive(Default, Clone, Copy)]
    pub struct ResponseFlags: u64 {}
}
