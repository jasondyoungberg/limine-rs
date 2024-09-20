//! Request structures

use core::{cell::UnsafeCell, ptr::NonNull};

use crate::{modules::InternalModule, paging, response::*, smp};

macro_rules! impl_base_fns {
    ($latest_revision:expr, $response:ty, $magic:expr, { $($(#[$attr:meta])* $field:ident: $val:expr),* $(,)? }) => {
        /// Create a new request with the latest revision.
        pub const fn new() -> Self {
            Self::with_revision($latest_revision)
        }

        /// Create a new request with the given revision.
        pub const fn with_revision(revision: u64) -> Self {
            Self {
                id: $magic,
                revision,
                response: Response::none(),
                $($(#[$attr])* $field: $val),*
            }
        }

        /// Get the ID of this request. This includes the magic number and the
        /// request-specific ID.
        pub fn id(&self) -> &[u64; 4] {
            &self.id
        }

        /// Get the revision of this request.
        pub fn revision(&self) -> u64 {
            self.revision
        }

        /// Get the response to this request, if it has been set.
        pub fn get_response(&self) -> Option<&$response> {
            self.response.get()
        }
        /// Get a mutable reference to the response to this request, if it has
        /// been set.
        ///
        /// Note that this method takes a mutable reference, so the request will
        /// have to be wrapped in a mutex or similar in order to use it.
        pub fn get_response_mut(&mut self) -> Option<&mut $response> {
            self.response.get_mut()
        }
    };
}

macro_rules! setter {
    ($(#[$attr:meta])* $ty:ty, $setter:ident, $with:ident, $field:ident) => {
        $(#[$attr])*
        /// This function operates in place.
        ///
        /// # Parameters
        #[doc = concat!("- ", stringify!($field), ": The new value for the field.")]
        pub fn $setter(&mut self, $field: impl Into<$ty>) {
            self.$field = $field.into();
        }
        $(#[$attr])*
        /// This function returns the new value.
        ///
        /// # Parameters
        #[doc = concat!("- ", stringify!($field), ": The new value for the field.")]
        pub const fn $with(mut self, $field: $ty) -> Self {
            self.$field = $field;
            self
        }
    };
}

macro_rules! magic {
    ($first:expr, $second:expr) => {
        [0xc7b1dd30df4c8b88, 0x0a82e883a194f07b, $first, $second]
    };
}

/// Marks the begining of the requests section
#[repr(transparent)]
pub struct RequestsStartMarker {
    id: [u64; 4],
}
impl RequestsStartMarker {
    /// Create a new request start marker
    pub const fn new() -> Self {
        Self {
            id: [
                0xf6b8f4b39de7d1ae,
                0xfab91a6940fcb9cf,
                0x785c6ed015d3e316,
                0x181e920a7852b9d9,
            ],
        }
    }
}

/// Marks the end of the requests section
#[repr(transparent)]
pub struct RequestsEndMarker {
    id: [u64; 2],
}
impl RequestsEndMarker {
    /// Create a new request end marker
    pub const fn new() -> Self {
        Self {
            id: [0xadc0e0531bb10d03, 0x9572709f31764c62],
        }
    }
}

#[repr(transparent)]
struct Response<T> {
    inner: UnsafeCell<Option<NonNull<T>>>,
}
unsafe impl<T: Sync> Sync for Response<T> {}
unsafe impl<T: Send> Send for Response<T> {}
impl<T> Response<T> {
    pub fn get(&self) -> Option<&T> {
        Some(unsafe { core::ptr::read_volatile(self.inner.get())?.as_ref() })
    }
    pub fn get_mut(&mut self) -> Option<&mut T> {
        Some(unsafe { core::ptr::read_volatile(self.inner.get())?.as_mut() })
    }
}
impl<T> Response<T> {
    pub const fn none() -> Self {
        Self {
            inner: UnsafeCell::new(None),
        }
    }
}

/// Request the name and version of the loading bootloader.
///
/// # Usage
/// ```rust
/// # use limine::{request::BootloaderInfoRequest, response::BootloaderInfoResponse, BaseRevision};
/// static BASE_REVISION: BaseRevision = BaseRevision::new();
///
/// // Request the bootloader info
/// static BOOTLOADER_INFO_REQUEST: BootloaderInfoRequest = BootloaderInfoRequest::new();
///
/// # fn dummy<'a>() -> Option<&'a BootloaderInfoResponse> {
/// // ...later, in our code
/// BOOTLOADER_INFO_REQUEST.get_response() // ...
/// # }
/// ```
#[repr(C)]
pub struct BootloaderInfoRequest {
    id: [u64; 4],
    revision: u64,
    response: Response<BootloaderInfoResponse>,
}
impl BootloaderInfoRequest {
    impl_base_fns!(
        0,
        BootloaderInfoResponse,
        magic!(0xf55038d8e2a1202f, 0x279426fcf5f59740),
        {}
    );
}

/// Request the type of the firmware.
///
/// # Usage
/// ```rust
/// # use limine::{request::FirmwareTypeRequest, response::FirmwareTypeResponse, BaseRevision};
/// static BASE_REVISION: BaseRevision = BaseRevision::new();
///
/// // Request the firmware type
/// static FIRMWARE_TYPE_REQUEST: FirmwareTypeRequest = FirmwareTypeRequest::new();
///
/// # fn dummy<'a>() -> Option<&'a FirmwareTypeResponse> {
/// // ...later, in our code
/// FIRMWARE_TYPE_REQUEST.get_response() // ...
/// # }
/// ```
#[repr(C)]
pub struct FirmwareTypeRequest {
    id: [u64; 4],
    revision: u64,
    response: Response<FirmwareTypeResponse>,
}
impl FirmwareTypeRequest {
    impl_base_fns!(
        0,
        FirmwareTypeResponse,
        magic!(0x8c2f75d90bef28a8, 0x7045a4688eac00c3),
        {}
    );
}

/// Request a differently-sized stack.
///
/// # Usage
/// ```
/// # use limine::{request::StackSizeRequest, response::StackSizeResponse, BaseRevision};
/// static BASE_REVISION: BaseRevision = BaseRevision::new();
///
/// // Request a 128 KiB stack
/// static STACK_SIZE_REQUEST: StackSizeRequest = StackSizeRequest::new().with_size(0x32000);
///
/// # fn dummy<'a>() -> Option<&'a StackSizeResponse> {
/// // ...later, in our code
/// STACK_SIZE_REQUEST.get_response() // ...
/// # }
/// ```
#[repr(C)]
pub struct StackSizeRequest {
    id: [u64; 4],
    revision: u64,
    response: Response<StackSizeResponse>,
    size: u64,
}
impl StackSizeRequest {
    impl_base_fns!(
        0,
        StackSizeResponse,
        magic!(0x224ef0460a8e8926, 0xe1cb0fc25f46ea3d),
        {
            size: 0,
        }
    );

    setter!(
        /// Set the requested stack size, in bytes.
        u64,
        set_size,
        with_size,
        size
    );

    /// Get the requested stack size, in bytes.
    pub fn size(&self) -> u64 {
        self.size
    }
}

/// Request information about the higher-half direct mapping.
///
/// # Usage
/// ```rust
/// # use limine::{request::HhdmRequest, response::HhdmResponse, BaseRevision};
/// static BASE_REVISION: BaseRevision = BaseRevision::new();
///
/// // Request the higher-half direct mapping
/// static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();
///
/// # fn dummy<'a>() -> Option<&'a HhdmResponse> {
/// // ...later, in our code
/// HHDM_REQUEST.get_response() // ...
/// # }
/// ```
#[repr(C)]
pub struct HhdmRequest {
    id: [u64; 4],
    revision: u64,
    response: Response<HhdmResponse>,
}
impl HhdmRequest {
    impl_base_fns!(
        0,
        HhdmResponse,
        magic!(0x48dcf1cb8ad2b852, 0x63984e959a98244b),
        {}
    );
}

/// Request a framebuffer for graphics output.
///
/// # Usage
/// ```rust
/// # use limine::{request::FramebufferRequest, response::FramebufferResponse, BaseRevision};
/// static BASE_REVISION: BaseRevision = BaseRevision::new();
///
/// // Request a framebuffer
/// static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();
///
/// # fn dummy<'a>() -> Option<&'a FramebufferResponse> {
/// // ...later, in our code
/// FRAMEBUFFER_REQUEST.get_response() // ...
/// # }
#[repr(C)]
pub struct FramebufferRequest {
    id: [u64; 4],
    revision: u64,
    response: Response<FramebufferResponse>,
}
impl FramebufferRequest {
    impl_base_fns!(
        0,
        FramebufferResponse,
        magic!(0x9d5827dcd881dd75, 0xa3148604f6fab11b),
        {}
    );
}

/// Request certain platform-dependent paging modes and flags to be set.
///
/// # Usage
/// ```rust
/// # use limine::{request::PagingModeRequest, response::PagingModeResponse, paging, BaseRevision};
/// static BASE_REVISION: BaseRevision = BaseRevision::new();
///
/// // Request a paging mode
/// #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))] // x86_64 and AArch64 share the same modes
/// static PAGING_MODE_REQUEST: PagingModeRequest = PagingModeRequest::new().with_mode(paging::Mode::FOUR_LEVEL);
///
/// #[cfg(target_arch = "riscv64")] // RISC-V has different modes
/// static PAGING_MODE_REQUEST: PagingModeRequest = PagingModeRequest::new().with_mode(paging::Mode::SV48);
///
/// # fn dummy<'a>() -> Option<&'a PagingModeResponse> {
/// // ...later, in our code
/// PAGING_MODE_REQUEST.get_response() // ...
/// # }
#[repr(C)]
pub struct PagingModeRequest {
    id: [u64; 4],
    revision: u64,
    response: Response<PagingModeResponse>,
    mode: paging::Mode,

    // Revision 1+
    max_mode: paging::Mode,
    min_mode: paging::Mode,
}
impl PagingModeRequest {
    impl_base_fns!(
        1,
        PagingModeResponse,
        magic!(0x95c1a0edab0944cb, 0xa4e5cb3842f7488a),
        {
            mode: paging::Mode::DEFAULT,
            max_mode: paging::Mode::DEFAULT,
            min_mode: paging::Mode::MIN,
        }
    );

    setter!(
        /// Set the requested paging mode. See [`Mode`](paging::Mode) for more
        /// information.
        paging::Mode,
        set_mode,
        with_mode,
        mode
    );
    setter!(
        /// Set the requested maximum paging mode. See [`Mode`](paging::Mode) for more
        /// information.
        paging::Mode,
        set_max_mode,
        with_max_mode,
        max_mode
    );
    setter!(
        /// Set the requested minimum paging mode. See [`Mode`](paging::Mode) for more
        /// information.
        paging::Mode,
        set_min_mode,
        with_min_mode,
        min_mode
    );

    /// Get the requested paging mode. See [`Mode`](paging::Mode) for more
    /// information.
    pub fn mode(&self) -> paging::Mode {
        self.mode
    }
    /// Get the requested maximum paging mode. See [`Mode`](paging::Mode) for more
    /// information.
    pub fn max_mode(&self) -> paging::Mode {
        self.max_mode
    }
    /// Get the requested minimum paging mode. See [`Mode`](paging::Mode) for more
    /// information.
    pub fn min_mode(&self) -> paging::Mode {
        self.min_mode
    }
}

/// Request the start of all other cores on the system, if they exist. Without
/// this request, non-bootstrap cores will be ignored.
///
/// # Usage
/// ```rust
/// # use limine::{request::SmpRequest, response::SmpResponse, BaseRevision};
/// static BASE_REVISION: BaseRevision = BaseRevision::new();
///
/// // Request that all other cores be started
/// static SMP_REQUEST: SmpRequest = SmpRequest::new();
///
/// # fn dummy<'a>() -> Option<&'a SmpResponse> {
/// // ...later, in our code
/// SMP_REQUEST.get_response() // ...
/// # }
/// ```
#[repr(C)]
pub struct SmpRequest {
    id: [u64; 4],
    revision: u64,
    response: Response<SmpResponse>,
    flags: smp::RequestFlags,
}
impl SmpRequest {
    impl_base_fns!(
        0,
        SmpResponse,
        magic!(0x95a67b819a1b857e, 0xa0b61b723b6a73e0),
        {
            flags: smp::RequestFlags::empty(),
        }
    );

    setter!(
        /// Set the SMP request flags. See [`RequestFlags`](smp::RequestFlags)
        /// for more information.
        smp::RequestFlags,
        set_flags,
        with_flags,
        flags
    );

    /// Get the SMP request flags. See [`RequestFlags`](smp::RequestFlags) for
    /// more information.
    pub fn flags(&self) -> smp::RequestFlags {
        self.flags
    }
}

/// Request limine's memory map. This may or may not be the same as the one
/// given by the BIOS/UEFI firmware. Entries are guaranteed to be in order of
/// their base address. Usable and bootloader-reclaimable memory regions will
/// never overlap, and will always be 4096-byte aligned. Other region types have
/// no such guarantees. Some holes may exist. Memory between 0x0-0x1000 is never
/// marked as usable.
///
/// # Usage
/// ```rust
/// # use limine::{request::MemoryMapRequest, response::MemoryMapResponse, BaseRevision};
/// static BASE_REVISION: BaseRevision = BaseRevision::new();
///
/// // Request a memory map
/// static MEMORY_MAP_REQUEST: MemoryMapRequest = MemoryMapRequest::new();
///
/// # fn dummy<'a>() -> Option<&'a MemoryMapResponse> {
/// // ...later, in our code
/// MEMORY_MAP_REQUEST.get_response() // ...
/// # }
/// ```
#[repr(C)]
pub struct MemoryMapRequest {
    id: [u64; 4],
    revision: u64,
    response: Response<MemoryMapResponse>,
}
impl MemoryMapRequest {
    impl_base_fns!(
        0,
        MemoryMapResponse,
        magic!(0x67cf3d9d378a806f, 0xe304acdfc50c3c62),
        {}
    );
}

/// Requests limine to use a specific function as the kernel entry point,
/// instead of the one specified in the ELF.
#[repr(C)]
pub struct EntryPointRequest {
    id: [u64; 4],
    revision: u64,
    response: Response<EntryPointResponse>,
    entry_point: extern "C" fn() -> !,
}
impl EntryPointRequest {
    impl_base_fns!(
        0,
        EntryPointResponse,
        magic!(0x13d86c035a1cd3e1, 0x2b0caa89d8f3026a),
        {
            entry_point: {
                extern "C" fn dummy() -> ! {
                    loop {
                        core::hint::spin_loop();
                    }
                }
                dummy
            },
        }
    );

    setter!(
        /// Set the entry point function. The function must never return.
        extern "C" fn() -> !,
        set_entry_point,
        with_entry_point,
        entry_point
    );

    /// Get the entry point function. The function must never return.
    pub fn entry_point(&self) -> extern "C" fn() -> ! {
        self.entry_point
    }
}

/// Request information about the loaded kernel file. See [`File`](crate::file::File)
/// for more information.
///
/// # Usage
/// ```rust
/// # use limine::{request::KernelFileRequest, response::KernelFileResponse, BaseRevision};
/// static BASE_REVISION: BaseRevision = BaseRevision::new();
///
/// // Request information about the kernel file
/// static KERNEL_FILE_REQUEST: KernelFileRequest = KernelFileRequest::new();
///
/// # fn dummy<'a>() -> Option<&'a KernelFileResponse> {
/// // ...later, in our code
/// KERNEL_FILE_REQUEST.get_response() // ...
/// # }
/// ```
#[repr(C)]
pub struct KernelFileRequest {
    id: [u64; 4],
    revision: u64,
    response: Response<KernelFileResponse>,
}
impl KernelFileRequest {
    impl_base_fns!(
        0,
        KernelFileResponse,
        magic!(0xad97e90e83f1ed67, 0x31eb5d1c5ff23b69),
        {}
    );
}

/// Request information about the loaded modules. See [`File`](crate::file::File) for
/// more information.
///
/// Additionally, with revision 1, this request can be used to specify
/// additional modules to be loaded without being specified in the configuration
/// file.
///
/// # Revisions
/// - Revision 0: Initial revision.
/// - Revision 1: added `internal_modules`
///
/// # Usage
/// ```rust
/// # use limine::{request::ModuleRequest, response::ModuleResponse, modules::*, BaseRevision};
/// static BASE_REVISION: BaseRevision = BaseRevision::new();
///
/// // Try to load our own module
/// static MODULE_REQUEST: ModuleRequest =
///    ModuleRequest::new().with_internal_modules(&[
///        &InternalModule::new().with_path(limine::cstr!("/path/to/a/module"))
///    ]);
///
/// # fn dummy<'a>() -> Option<&'a ModuleResponse> {
/// // ...later, in our code
/// MODULE_REQUEST.get_response() // ...
/// # }
/// ```
#[repr(C)]
pub struct ModuleRequest {
    id: [u64; 4],
    revision: u64,
    response: Response<ModuleResponse>,

    // Revision 1+
    internal_module_ct: u64,
    internal_modules: *const *const InternalModule,
}
unsafe impl Sync for ModuleRequest {}
unsafe impl Send for ModuleRequest {}
impl ModuleRequest {
    impl_base_fns!(
        1,
        ModuleResponse,
        magic!(0x3e7e279702be32af, 0xca1c4f3bd1280cee),
        {
            internal_module_ct: 0,
            internal_modules: core::ptr::null(),
        }
    );

    /// Seet the internal modules to be loaded. Only available on revision 1+.
    /// This function operates in place.
    ///
    /// # Parameters
    /// - `modules`: The new value of the field.
    pub fn set_internal_modules(&mut self, modules: &'static [&'static InternalModule]) {
        self.internal_module_ct = modules.len() as u64;
        self.internal_modules = modules.as_ptr().cast();
    }

    /// Set the internal modules to be loaded. Only available on revision 1+.
    /// This function returns the new value.
    ///
    /// # Parameters
    /// - `modules`: The new value of the field.
    pub const fn with_internal_modules(
        mut self,
        modules: &'static [&'static InternalModule],
    ) -> Self {
        self.internal_module_ct = modules.len() as u64;
        self.internal_modules = modules.as_ptr().cast();
        self
    }

    /// Get the internal modules to be loaded. Only available on revision 1.
    pub fn internal_modules(&self) -> &[&InternalModule] {
        unsafe {
            core::slice::from_raw_parts(
                self.internal_modules.cast(),
                self.internal_module_ct as usize,
            )
        }
    }
}

/// Request the RSDP address.
///
/// # Usage
/// ```rust
/// # use limine::{request::RsdpRequest, response::RsdpResponse, BaseRevision};
/// static BASE_REVISION: BaseRevision = BaseRevision::new();
///
/// // Request the RSDP address
/// static Rsdp_REQUEST: RsdpRequest = RsdpRequest::new();
///
/// # fn dummy<'a>() -> Option<&'a RsdpResponse> {
/// // ...later, in our code
/// Rsdp_REQUEST.get_response() // ...
/// # }
/// ```
#[repr(C)]
pub struct RsdpRequest {
    id: [u64; 4],
    revision: u64,
    response: Response<RsdpResponse>,
}
impl RsdpRequest {
    impl_base_fns!(
        0,
        RsdpResponse,
        magic!(0xc5e77b6b397e7b43, 0x27637845accdcf3c),
        {}
    );
}

/// Request the address of the SMBIOS table.
///
/// # Usage
/// ```rust
/// # use limine::{request::SmbiosRequest, response::SmbiosResponse, BaseRevision};
/// static BASE_REVISION: BaseRevision = BaseRevision::new();
///
/// // Request the SMBIOS table address
/// static SMBIOS_REQUEST: SmbiosRequest = SmbiosRequest::new();
///
/// # fn dummy<'a>() -> Option<&'a SmbiosResponse> {
/// // ...later, in our code
/// SMBIOS_REQUEST.get_response() // ...
/// # }
/// ```
#[repr(C)]
pub struct SmbiosRequest {
    id: [u64; 4],
    revision: u64,
    response: Response<SmbiosResponse>,
}
impl SmbiosRequest {
    impl_base_fns!(
        0,
        SmbiosResponse,
        magic!(0x9e9046f11e095391, 0xaa4a520fefbde5ee),
        {}
    );
}

/// Request the address of the UEFI system table.
///
/// # Usage
/// ```rust
/// # use limine::{request::EfiSystemTableRequest, response::EfiSystemTableResponse, BaseRevision};
/// static BASE_REVISION: BaseRevision = BaseRevision::new();
///
/// // Request the UEFI system table address
/// static EFI_SYSTEM_TABLE_REQUEST: EfiSystemTableRequest = EfiSystemTableRequest::new();
///
/// # fn dummy<'a>() -> Option<&'a EfiSystemTableResponse> {
/// // ...later, in our code
/// EFI_SYSTEM_TABLE_REQUEST.get_response() // ...
/// # }
/// ```
#[repr(C)]
pub struct EfiSystemTableRequest {
    id: [u64; 4],
    revision: u64,
    response: Response<EfiSystemTableResponse>,
}
impl EfiSystemTableRequest {
    impl_base_fns!(
        0,
        EfiSystemTableResponse,
        magic!(0x5ceba5163eaaf6d6, 0x0a6981610cf65fcc),
        {}
    );
}

/// Request the address of the UEFI memory map.
///
/// # Usage
/// ```rust
/// # use limine::{request::EfiMemoryMapRequest, response::EfiMemoryMapResponse, BaseRevision};
/// static BASE_REVISION: BaseRevision = BaseRevision::new();
///
/// // Request the UEFI memory map address
/// static EFI_MEMORY_MAP_REQUEST: EfiMemoryMapRequest = EfiMemoryMapRequest::new();
///
/// # fn dummy<'a>() -> Option<&'a EfiMemoryMapResponse> {
/// // ...later, in our code
/// EFI_MEMORY_MAP_REQUEST.get_response() // ...
/// # }
/// ```
#[repr(C)]
pub struct EfiMemoryMapRequest {
    id: [u64; 4],
    revision: u64,
    response: Response<EfiMemoryMapResponse>,
}
impl EfiMemoryMapRequest {
    impl_base_fns!(
        0,
        EfiMemoryMapResponse,
        magic!(0x7df62a431d6872d5, 0xa4fcdfb3e57306c8),
        {}
    );
}

/// Request the boot time in seconds
///
/// # Usage
/// ```rust
/// # use limine::{request::BootTimeRequest, response::BootTimeResponse, BaseRevision};
/// static BASE_REVISION: BaseRevision = BaseRevision::new();
///
/// // Request the boot time
/// static BOOT_TIME_REQUEST: BootTimeRequest = BootTimeRequest::new();
///
/// # fn dummy<'a>() -> Option<&'a BootTimeResponse> {
/// // ...later, in our code
/// BOOT_TIME_REQUEST.get_response() // ...
/// # }
/// ```
#[repr(C)]
pub struct BootTimeRequest {
    id: [u64; 4],
    revision: u64,
    response: Response<BootTimeResponse>,
}
impl BootTimeRequest {
    impl_base_fns!(
        0,
        BootTimeResponse,
        magic!(0x502746e184c088aa, 0xfbc5ec83e6327893),
        {}
    );
}

/// Request the base address of the kernel code, in virtual and physical space.
///
/// # Usage
/// ```rust
/// # use limine::{request::KernelAddressRequest, response::KernelAddressResponse, BaseRevision};
/// static BASE_REVISION: BaseRevision = BaseRevision::new();
///
/// // Request the kernel address
/// static KERNEL_ADDRESS_REQUEST: KernelAddressRequest = KernelAddressRequest::new();
///
/// # fn dummy<'a>() -> Option<&'a KernelAddressResponse> {
/// // ...later, in our code
/// KERNEL_ADDRESS_REQUEST.get_response() // ...
/// # }
/// ```
#[repr(C)]
pub struct KernelAddressRequest {
    id: [u64; 4],
    revision: u64,
    response: Response<KernelAddressResponse>,
}
impl KernelAddressRequest {
    impl_base_fns!(
        0,
        KernelAddressResponse,
        magic!(0x71ba76863cc55f63, 0xb2644a48c516a487),
        {}
    );
}

/// Request the address of the device-tree blob, if one is present.
///
/// # Usage
/// ```rust
/// # use limine::{request::DeviceTreeBlobRequest, response::DeviceTreeBlobResponse, BaseRevision};
/// static BASE_REVISION: BaseRevision = BaseRevision::new();
///
/// // Request the device-tree blob address
/// static DEVICE_TREE_BLOB_REQUEST: DeviceTreeBlobRequest = DeviceTreeBlobRequest::new();
///
/// # fn dummy<'a>() -> Option<&'a DeviceTreeBlobResponse> {
/// // ...later, in our code
/// DEVICE_TREE_BLOB_REQUEST.get_response() // ...
/// # }
/// ```
#[repr(C)]
pub struct DeviceTreeBlobRequest {
    id: [u64; 4],
    revision: u64,
    response: Response<DeviceTreeBlobResponse>,
}
impl DeviceTreeBlobRequest {
    impl_base_fns!(
        0,
        DeviceTreeBlobResponse,
        magic!(0xb40ddb48fb54bac7, 0x545081493f81ffb7),
        {}
    );
}
