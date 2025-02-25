//! Responses to [requests](crate::request).

use core::{
    ffi::{c_char, c_void, CStr},
    num::NonZeroUsize,
    time::Duration,
};

use crate::{
    file,
    firmware_type::FirmwareType,
    framebuffer::{Framebuffer, RawFramebuffer},
    memory_map, mp,
    paging::Mode,
};

macro_rules! impl_base_fns {
    () => {
        /// Returns the revision of the response.
        pub fn revision(&self) -> u64 {
            self.revision
        }
    };
}

/// A response to a [bootloader info
/// request](crate::request::BootloaderInfoRequest).
#[repr(C)]
pub struct BootloaderInfoResponse {
    revision: u64,
    name: *const c_char,
    version: *const c_char,
}
unsafe impl Sync for BootloaderInfoResponse {}
unsafe impl Send for BootloaderInfoResponse {}
impl BootloaderInfoResponse {
    impl_base_fns!();

    /// Returns the name of the loading bootloader.
    pub fn name(&self) -> &str {
        unsafe { CStr::from_ptr(self.name) }.to_str().unwrap()
    }

    /// Returns the version of the loading bootloader.
    pub fn version(&self) -> &str {
        unsafe { CStr::from_ptr(self.version) }.to_str().unwrap()
    }
}

/// A response to a [firmware type request
/// ](crate::request::FirmwareTypeRequest).
#[repr(C)]
pub struct FirmwareTypeResponse {
    revision: u64,
    firmware_type: FirmwareType,
}
impl FirmwareTypeResponse {
    impl_base_fns!();

    /// Returns the firmware type.
    pub fn firmware_type(&self) -> FirmwareType {
        self.firmware_type
    }
}

/// A response to a [stack size request](crate::request::StackSizeRequest). This
/// response has no fields. If it is provided, the bootloader complied with the
/// request.
#[repr(C)]
pub struct StackSizeResponse {
    revision: u64,
}
impl StackSizeResponse {
    impl_base_fns!();
}

/// A response to a [higher-half direct map
/// request](crate::request::HhdmRequest).
#[repr(C)]
pub struct HhdmResponse {
    revision: u64,
    offset: u64,
}
impl HhdmResponse {
    impl_base_fns!();

    /// Returns the offset of the higher-half direct map. This can be used to
    /// convert physical addresses to virtual addresses, and the same in
    /// reverse, as long as the bootloader's page tables are still in use.
    ///
    /// # Examples
    /// Convert physical to virtual:
    /// ```rust
    /// # let offset = 42;
    /// # let phys_addr = 42;
    /// let virt_addr = phys_addr + offset;
    /// ```
    /// Convert virtual† to physical:
    /// ```rust
    /// # let offset = 42;
    /// # let virt_addr = 42;
    /// let phys_addr = virt_addr - offset;
    /// ```
    ///
    /// † Note that this only works if the virtual address is in the higher-half
    ///   direct map already. This is true of any virtual address returned in
    ///   any response. However, this is not true of addresses within executable
    ///   code. To convert an address within executable code, you must use the
    ///   [executable address request](crate::request::ExecutableAddressRequest).
    pub fn offset(&self) -> u64 {
        self.offset
    }
}

/// A response to a [framebuffer request](crate::request::FramebufferRequest).
#[repr(C)]
pub struct FramebufferResponse {
    revision: u64,
    framebuffer_ct: u64,
    framebuffers: *const *const RawFramebuffer,
}
unsafe impl Sync for FramebufferResponse {}
unsafe impl Send for FramebufferResponse {}
impl FramebufferResponse {
    impl_base_fns!();

    /// Returns an iterator over the found framebuffers. See [`Framebuffer`] for
    /// more information.
    pub fn framebuffers(&self) -> impl Iterator<Item = Framebuffer<'_>> {
        (unsafe { core::slice::from_raw_parts(self.framebuffers, self.framebuffer_ct as usize) })
            .iter()
            .map(|&fb| Framebuffer::new(self.revision, unsafe { &*fb }))
    }
}

/// A response to a [paging mode request](crate::request::PagingModeRequest).
#[repr(C)]
pub struct PagingModeResponse {
    revision: u64,
    mode: Mode,
}
impl PagingModeResponse {
    impl_base_fns!();

    /// Returns mode that was enabled by the bootloader. See [`Mode`] for more
    /// information.
    pub fn mode(&self) -> Mode {
        self.mode
    }
}

#[deprecated(since = "0.4.0", note = "please use `MpResponse` instead")]
/// A response to a [smp request](crate::request::SmpRequest). This response
/// contains information about the boot processor and all other processors.
pub type SmpResponse = MpResponse;

/// A response to a [mp request](crate::request::MpRequest). This response
/// contains information about the boot processor and all other processors.
#[repr(C)]
pub struct MpResponse {
    #[cfg(not(target_arch = "loongarch64"))]
    revision: u64,
    #[cfg(not(target_arch = "loongarch64"))]
    flags: mp::ResponseFlags,
    #[cfg(target_arch = "x86_64")]
    bsp_lapic_id: u32,
    #[cfg(target_arch = "aarch64")]
    bsp_mpidr: u64,
    #[cfg(target_arch = "riscv64")]
    bsp_hartid: u64,
    cpu_ct: u64,
    cpus: *mut *mut mp::Cpu,
}
unsafe impl Sync for MpResponse {}
unsafe impl Send for MpResponse {}
impl MpResponse {
    #[cfg(not(target_arch = "loongarch64"))]
    impl_base_fns!();

    /// Returns the flags that were enabled by the bootloader. See
    /// [`ResponseFlags`](mp::ResponseFlags) for more information.
    #[cfg(not(target_arch = "loongarch64"))]
    pub fn flags(&self) -> mp::ResponseFlags {
        self.flags
    }

    /// Returns the local APIC ID of the boot processor. This is only available
    /// on x86_64.
    #[cfg(target_arch = "x86_64")]
    pub fn bsp_lapic_id(&self) -> u32 {
        self.bsp_lapic_id
    }

    /// Returns the value of the MPIDR on the boot processor. This is only
    /// available on aarch64.
    #[cfg(target_arch = "aarch64")]
    pub fn bsp_mpidr(&self) -> u64 {
        self.bsp_mpidr
    }

    /// Returns the hart ID of the boot processor. This is only available on
    /// riscv64.
    #[cfg(target_arch = "riscv64")]
    pub fn bsp_hartid(&self) -> u64 {
        self.bsp_hartid
    }

    /// Returns a slice of found CPUs. See [`Cpu`](mp::Cpu) for more information.
    pub fn cpus(&self) -> &[&mp::Cpu] {
        unsafe { core::slice::from_raw_parts(self.cpus.cast(), self.cpu_ct as usize) }
    }

    /// Returns a mutable slice of found CPUs. See [`Cpu`](mp::Cpu) for more information.
    /// Note that this function takes `&mut self`, so the response will likely
    /// need to be wrapped in a `Mutex` or similar. It is provided so that the
    /// `extra` field on each CPU can be set.
    pub fn cpus_mut(&mut self) -> &mut [&mut mp::Cpu] {
        unsafe { core::slice::from_raw_parts_mut(self.cpus.cast(), self.cpu_ct as usize) }
    }
}

/// A response to a [memory map request](crate::request::MemoryMapRequest).
#[repr(C)]
pub struct MemoryMapResponse {
    revision: u64,
    entry_ct: u64,
    entries: *mut *mut memory_map::Entry,
}
unsafe impl Sync for MemoryMapResponse {}
unsafe impl Send for MemoryMapResponse {}
impl MemoryMapResponse {
    impl_base_fns!();

    /// Returns a slice of found memory map entries. See
    /// [`Entry`](memory_map::Entry) for more information.
    pub fn entries(&self) -> &[&memory_map::Entry] {
        unsafe { core::slice::from_raw_parts(self.entries.cast(), self.entry_ct as usize) }
    }

    /// Returns a mutable slice of found memory map entries. See
    /// [`Entry`](memory_map::Entry) for more information. Note that this
    /// function takes `&mut self`, so the response will likely need to be
    /// wrapped in a `Mutex` or similar.
    pub fn entries_mut(&mut self) -> &mut [&mut memory_map::Entry] {
        unsafe { core::slice::from_raw_parts_mut(self.entries.cast(), self.entry_ct as usize) }
    }
}

/// A response to a [executable file request](crate::request::ExecutableFileRequest).
#[repr(C)]
pub struct EntryPointResponse {
    revision: u64,
}
impl EntryPointResponse {
    impl_base_fns!();
}

#[deprecated(since = "0.4.0", note = "please use `ExecutableFileResponse` instead")]
/// A response to a [kernel file request](crate::request::KernelFileRequest).
pub type KernelFileResponse = ExecutableFileResponse;

/// A response to a [executable file request](crate::request::ExecutableFileRequest).
#[repr(C)]
pub struct ExecutableFileResponse {
    revision: u64,
    file: *const file::File,
}
unsafe impl Sync for ExecutableFileResponse {}
unsafe impl Send for ExecutableFileResponse {}
impl ExecutableFileResponse {
    impl_base_fns!();

    /// Returns the executable file. See [`File`](file::File) for more information.
    pub fn file(&self) -> &file::File {
        unsafe { &*self.file }
    }
}

/// A response to a [module request](crate::request::ModuleRequest).
#[repr(C)]
pub struct ModuleResponse {
    revision: u64,
    module_ct: u64,
    modules: *const *const file::File,
}
unsafe impl Sync for ModuleResponse {}
unsafe impl Send for ModuleResponse {}
impl ModuleResponse {
    impl_base_fns!();

    /// Returns a slice of loaded modules. See [`File`](file::File) for more
    /// information.
    pub fn modules(&self) -> &[&file::File] {
        unsafe { core::slice::from_raw_parts(self.modules.cast(), self.module_ct as usize) }
    }
}

/// A response to a [rsdp request](crate::request::RsdpRequest).
#[repr(C)]
pub struct RsdpResponse {
    revision: u64,
    address: usize,
}
unsafe impl Sync for RsdpResponse {}
unsafe impl Send for RsdpResponse {}
impl RsdpResponse {
    impl_base_fns!();

    /// Returns the address of the RSDP table in the ACPI.
    pub fn address(&self) -> usize {
        self.address
    }
}

/// A response to a [smbios request](crate::request::SmbiosRequest).
#[repr(C)]
pub struct SmbiosResponse {
    revision: u64,
    entry_32: Option<NonZeroUsize>,
    entry_64: Option<NonZeroUsize>,
}
unsafe impl Sync for SmbiosResponse {}
unsafe impl Send for SmbiosResponse {}
impl SmbiosResponse {
    impl_base_fns!();

    /// Returns the physical address of the SMBIOS 32-bit entry point, if it exists.
    pub fn entry_32(&self) -> Option<NonZeroUsize> {
        self.entry_32
    }
    /// Returns the physical address of the SMBIOS 64-bit entry point, if it exists.
    pub fn entry_64(&self) -> Option<NonZeroUsize> {
        self.entry_64
    }
}

/// A response to a [system table request](crate::request::EfiSystemTableRequest).
#[repr(C)]
pub struct EfiSystemTableResponse {
    revision: u64,
    address: usize,
}
unsafe impl Sync for EfiSystemTableResponse {}
unsafe impl Send for EfiSystemTableResponse {}
impl EfiSystemTableResponse {
    impl_base_fns!();

    /// Returns the address of the EFI system table.
    pub fn address(&self) -> usize {
        self.address
    }
}

/// A response to a [memory map request](crate::request::EfiMemoryMapRequest).
#[repr(C)]
pub struct EfiMemoryMapResponse {
    revision: u64,
    memmap: *const c_void,
    memmap_size: u64,
    desc_size: u64,
    desc_version: u32,
}
unsafe impl Sync for EfiMemoryMapResponse {}
unsafe impl Send for EfiMemoryMapResponse {}
impl EfiMemoryMapResponse {
    impl_base_fns!();

    /// Returns the address of the EFI memory map.
    pub fn memmap(&self) -> *const () {
        self.memmap.cast()
    }
    /// Returns the size of the EFI memory map.
    pub fn memmap_size(&self) -> u64 {
        self.memmap_size
    }

    /// Returns the size of each EFI memory map entry.
    pub fn desc_size(&self) -> u64 {
        self.desc_size
    }
    /// Returns the version of each EFI memory map entry.
    pub fn desc_version(&self) -> u32 {
        self.desc_version
    }
}

#[deprecated(since = "0.4.0", note = "please use `DateAtBootResponse` instead")]
/// A response to a [boot time request](crate::request::BootTimeRequest).
pub type BootTimeResponse = DateAtBootResponse;

/// A response to a [date at boot request](crate::request::DateAtBootRequest).
#[repr(C)]
pub struct DateAtBootResponse {
    revision: u64,
    timestamp: i64,
}
impl DateAtBootResponse {
    impl_base_fns!();

    /// Returns unix timestamp, as read from the system RTC.
    pub fn timestamp(&self) -> Duration {
        Duration::from_secs(self.timestamp as u64)
    }

    #[deprecated(
        since = "0.4.0",
        note = "please use `DateAtBootResponse::timestamp` instead"
    )]
    /// Returns the boot time in seconds, as read from the system RTC.
    pub fn boot_time(&self) -> Duration {
        self.timestamp()
    }
}

#[deprecated(
    since = "0.4.0",
    note = "please use `ExecutableAddressResponse` instead"
)]
/// A response to a [kernel address request](crate::request::KernelAddressRequest).
///
/// This can be used to convert a virtual address within the kernel to a
/// physical address like so:
/// ```rust
/// # let virt_addr = 42;
/// # let virtual_base = 42;
/// # let physical_base = 42;
/// let phys_addr = virt_addr - virtual_base + physical_base;
/// ````
pub type KernelAddressResponse = ExecutableAddressResponse;

/// A response to a [executable address request](crate::request::ExecutableAddressRequest).
///
/// This can be used to convert a virtual address within the executable to a
/// physical address like so:
/// ```rust
/// # let virt_addr = 42;
/// # let virtual_base = 42;
/// # let physical_base = 42;
/// let phys_addr = virt_addr - virtual_base + physical_base;
/// ````
#[repr(C)]
pub struct ExecutableAddressResponse {
    revision: u64,
    physical_base: u64,
    virtual_base: u64,
}
impl ExecutableAddressResponse {
    impl_base_fns!();

    /// Returns the base address of the executable in physical memory.
    pub fn physical_base(&self) -> u64 {
        self.physical_base
    }
    /// Returns the base address of the executable in virtual memory.
    pub fn virtual_base(&self) -> u64 {
        self.virtual_base
    }
}

/// A response to a [executable address request](crate::request::ExecutableAddressRequest).
///
/// This can be used to convert a virtual address within the executable to a
/// physical address like so:
/// ```rust
/// # let virt_addr = 42;
/// # let virtual_base = 42;
/// # let physical_base = 42;
/// let phys_addr = virt_addr - virtual_base + physical_base;
/// ````
#[repr(C)]
pub struct ExecutableCmdlineResponse {
    revision: u64,
    cmdline: *const c_char,
}
unsafe impl Sync for ExecutableCmdlineResponse {}
unsafe impl Send for ExecutableCmdlineResponse {}
impl ExecutableCmdlineResponse {
    impl_base_fns!();

    /// Returns the base address of the executable in physical memory.
    pub fn cmdline(&self) -> &CStr {
        unsafe { CStr::from_ptr(self.cmdline) }
    }
}

/// A response to a [device tree blob request](crate::request::DeviceTreeBlobRequest).
#[repr(C)]
pub struct DeviceTreeBlobResponse {
    revision: u64,
    dtb_ptr: *const c_void,
}
unsafe impl Sync for DeviceTreeBlobResponse {}
unsafe impl Send for DeviceTreeBlobResponse {}
impl DeviceTreeBlobResponse {
    impl_base_fns!();

    /// Returns the address of the device tree blob.
    pub fn dtb_ptr(&self) -> *const () {
        self.dtb_ptr.cast()
    }
}

/// A response to a [bsp hardid request](crate::request::BspHartidRequest).
#[cfg(target_arch = "riscv64")]
#[repr(C)]
pub struct BspHartidResponse {
    revision: u64,
    bsp_hartid: u64,
}
#[cfg(target_arch = "riscv64")]
impl BspHartidResponse {
    impl_base_fns!();

    /// Get the Hard ID of the boot processor.
    /// This is the same as [MpResponse::bsp_hartid], but doesn't boot up the other APs
    pub fn bsp_hartid(&self) -> u64 {
        self.bsp_hartid
    }
}
