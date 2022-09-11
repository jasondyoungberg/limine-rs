//! Rust crate for parsing the limine boot protocol structures.
//!
//! ## Resources
//! * [Limine Boot Protocol Specification](https://github.com/limine-bootloader/limine/blob/trunk/PROTOCOL.md)

#![no_std]
#![feature(const_nonnull_new)]

#[cfg(feature = "requests-section")]
pub use limine_proc::*;

use core::cell::UnsafeCell;
use core::ffi::{c_char, CStr};
use core::fmt::Debug;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;

#[derive(Debug)]
#[repr(transparent)]
pub struct LimineEntryPoint(*const ());

/// [`NonNull`] with the dereference traits implemented.
#[repr(transparent)]
pub struct NonNullPtr<T> {
    ptr: NonNull<T>,
    // This marker does not affect the variance but is required for
    // dropck to undestand that we logically own a `T`.
    //
    // TODO: Use `Unique<T>` when stabalized!
    _phantom: PhantomData<T>,
}

impl<T> NonNullPtr<T> {
    pub fn as_ptr(&self) -> *mut T {
        self.ptr.as_ptr()
    }
}

impl<T> Deref for NonNullPtr<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: We have shared reference to the data.
        unsafe { self.ptr.as_ref() }
    }
}

impl<T> DerefMut for NonNullPtr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: We have exclusive reference to the data.
        unsafe { self.ptr.as_mut() }
    }
}

impl<T: Debug> Debug for NonNullPtr<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let value: &T = &*self;

        f.debug_tuple("NonNullPtr")
            .field(&format_args!("{:#x?}", value))
            .finish()
    }
}

#[repr(transparent)]
pub struct LiminePtr<T> {
    ptr: Option<NonNull<T>>,
    // This marker does not affect the variance but is required for
    // dropck to undestand that we logically own a `T`.
    //
    // TODO: Use `Unique<T>` when stabalized!
    _phantom: PhantomData<T>,
}

impl<T> LiminePtr<T> {
    const DEFAULT: LiminePtr<T> = Self {
        ptr: None,
        _phantom: PhantomData,
    };

    #[inline]
    pub fn as_ptr(&self) -> Option<*mut T> {
        Some(self.ptr?.as_ptr())
    }

    #[inline]
    pub fn get<'a>(&self) -> Option<&'a T> {
        // SAFETY: According to the specication the bootloader provides
        // a aligned pointer and there is no public API to construct a [`LiminePtr`]
        // so, its safe to assume that the [`NonNull::as_ref`] are applied. If not,
        // its the bootloader's fault that they have violated the
        // specification!.
        //
        // Also, we have a shared reference to the data and there is no
        // legal way to mutate it, unless through [`LiminePtr::as_ptr`]
        // (requires pointer dereferencing which is unsafe) or [`LiminePtr::get_mut`]
        // (requires exclusive access to the [`LiminePtr`]).
        self.ptr.map(|e| unsafe { e.as_ref() })
    }

    #[inline]
    pub fn get_mut<'a>(&mut self) -> Option<&'a mut T> {
        // SAFETY: Check the safety for [`LiminePtr::get`] and we have
        // exclusive access to the data.
        self.ptr.as_mut().map(|e| unsafe { e.as_mut() })
    }
}

impl LiminePtr<c_char> {
    /// Converts the limine string pointer into a rust string.
    pub fn to_str(&self) -> Option<&CStr> {
        // SAFETY: According to the limine specification, the pointer points
        // to a valid C string with a NULL terminator of size less than
        // `isize::MAX`. Also we know that the `LiminePtr` is a valid C string,
        // because it has a `T` of `c_char`. See the [`LiminePtr::get`] for more
        // details.
        unsafe { Some(CStr::from_ptr(self.as_ptr()?)) }
    }
}

impl LiminePtr<LimineEntryPoint> {
    #[inline]
    pub const fn new(entry_point: fn() -> !) -> Self {
        Self {
            ptr: NonNull::new(entry_point as *mut _),
            _phantom: PhantomData,
        }
    }
}

impl<T: Debug> Debug for LiminePtr<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("LiminePtr")
            .field(&format_args!("{:#x?}", self.get()))
            .finish()
    }
}

unsafe impl<T: Sync> Sync for LiminePtr<T> {}

type ArrayPtr<T> = NonNullPtr<NonNullPtr<T>>;

impl<T> ArrayPtr<T> {
    fn into_slice<'a>(&'a self, len: usize) -> &'a [NonNullPtr<T>] {
        // SAFETY: We have shared reference to the array.
        unsafe { core::slice::from_raw_parts(self.as_ptr(), len) }
    }

    fn into_slice_mut<'a>(&'a mut self, len: usize) -> &'a mut [NonNullPtr<T>] {
        // SAFETY: We have exculusive access to the array.
        unsafe { core::slice::from_raw_parts_mut(self.as_ptr(), len) }
    }
}

/// Used to create the limine request struct.
macro_rules! make_struct {
    (
        $(#[$meta:meta])*
        struct $name:ident: [$id1:expr, $id2:expr] => $response_ty:ty {
            $($(#[$field_meta:meta])* $field_name:ident : $field_ty:ty = $field_default:expr),*
        };
    ) => {
        $(#[$meta])*
        #[repr(C)]
        #[derive(Debug)]
        pub struct $name {
            id: [u64; 4],
            revision: u64,

            // NOTE: The response is required to be wrapped inside an unsafe cell, since
            // by default the response is set to NULL and when the compiler does not see
            // any writes to the field, it is free to assume that the response is NULL. In
            // our situation the bootloader mutates the field and we need to ensure that
            // the compiler does not optimize the read away.
            response: UnsafeCell<LiminePtr<$response_ty>>,
            $(pub $field_name: $field_ty),*
        }

        impl $name {
            // NOTE: The request ID is composed of 4 64-bit wide unsigned integers but the first
            // two remain constant. This is refered as `LIMINE_COMMON_MAGIC` in the limine protocol
            // header.
            pub const ID: [u64; 4] = [0xc7b1dd30df4c8b88, 0x0a82e883a194f07b, $id1, $id2];

            pub const fn new(revision: u64) -> Self {
                Self {
                    id: Self::ID,
                    revision,

                    response: UnsafeCell::new(LiminePtr::DEFAULT),
                    $($field_name: $field_default),*
                }
            }

            pub fn get_response(&self) -> LiminePtr<$response_ty> {
                unsafe { core::ptr::read_volatile(self.response.get()) }
            }

            // generate a getter method for each field:
            $($(#[$field_meta])* pub const fn $field_name(mut self, value: $field_ty) -> Self {
                self.$field_name = value;
                self
            })*
        }

        // maker trait implementations for limine request struct:
        unsafe impl Sync for $name {}
    };
}

// misc structures:

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct LimineUuid {
    pub a: u32,
    pub b: u16,
    pub c: u16,
    pub d: [u8; 8],
}

#[cfg(feature = "into-uuid")]
impl From<LimineUuid> for uuid::Uuid {
    fn from(lim: LimineUuid) -> Self {
        Self::from_fields(lim.a, lim.b, lim.c, &lim.d)
    }
}

#[cfg(feature = "into-uuid")]
impl From<uuid::Uuid> for LimineUuid {
    fn from(uuid: uuid::Uuid) -> Self {
        let (a, b, c, d) = uuid.as_fields();
        Self {
            a,
            b,
            c,
            d: d.clone(),
        }
    }
}

#[derive(Debug)]
pub struct LimineFile {
    /// Revision of this structure.
    pub revision: u64,
    /// The address of the file.
    pub base: LiminePtr<u8>,
    /// The size of the file.
    pub length: u64,
    /// The path of the file within the volume, with a leading slash.
    pub path: LiminePtr<c_char>,
    /// A command line associated with the file.
    pub cmdline: LiminePtr<c_char>,
    /// Type of media file resides on.
    pub media_type: u64,
    pub unused: u32,
    /// If non-0, this is the IP of the TFTP server the file was loaded from.
    pub tftp_ip: u32,
    /// Likewise, but port.
    pub tftp_port: u32,
    /// 1-based partition index of the volume from which the file was loaded. If 0, it
    /// means invalid or unpartitioned.
    pub partition_index: u32,
    /// If non-0, this is the ID of the disk the file was loaded from as reported in its MBR.
    pub mbr_disk_id: u32,
    /// If non-0, this is the UUID of the disk the file was loaded from as reported in its GPT.
    pub gpt_disk_uuid: LimineUuid,
    /// If non-0, this is the UUID of the partition the file was loaded from as reported in the GPT.
    pub gpt_part_uuid: LimineUuid,
    /// If non-0, this is the UUID of the filesystem of the partition the file was loaded from.
    pub part_uuid: LimineUuid,
}

// boot info request tag:
#[repr(C)]
#[derive(Debug)]
pub struct LimineBootInfoResponse {
    pub revision: u64,
    /// Null-terminated string containing the name of the bootloader.
    pub name: LiminePtr<c_char>,
    /// Null-terminated string containg the version of the bootloader.
    pub version: LiminePtr<c_char>,
}

make_struct!(
    struct LimineBootInfoRequest: [0xf55038d8e2a1202f, 0x279426fcf5f59740] => LimineBootInfoResponse {};
);

// stack size request tag:
#[repr(C)]
#[derive(Debug)]
pub struct LimineStackSizeResponse {
    pub revision: u64,
}

make_struct!(
    struct LimineStackSizeRequest: [0x224ef0460a8e8926, 0xe1cb0fc25f46ea3d] => LimineStackSizeResponse {
        /// The requested stack size (also used for SMP processors).
        stack_size: u64 = 0
    };
);

// HHDM request tag:
#[repr(C)]
#[derive(Debug)]
pub struct LimineHhdmResponse {
    pub revision: u64,
    /// The virtual address offset of the beginning of the higher half direct map.
    pub offset: u64,
}

make_struct!(
    struct LimineHhdmRequest: [0x48dcf1cb8ad2b852, 0x63984e959a98244b] => LimineHhdmResponse {};
);

// framebuffer request tag:
#[repr(C)]
#[derive(Debug)]
pub struct LimineFramebuffer {
    pub address: LiminePtr<u8>,
    pub width: u64,
    pub height: u64,
    pub pitch: u64,
    pub bpp: u16,
    pub memory_model: u8,
    pub red_mask_size: u8,
    pub red_mask_shift: u8,
    pub green_mask_size: u8,
    pub green_mask_shift: u8,
    pub blue_mask_size: u8,
    pub blue_mask_shift: u8,
    pub reserved: [u8; 7],
    pub edid_size: u64,
    pub edid: LiminePtr<u8>,
}

impl LimineFramebuffer {
    /// Returns the size of the framebuffer.
    pub fn size(&self) -> usize {
        self.pitch as usize * self.height as usize * (self.bpp as usize / 8)
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct LimineFramebufferResponse {
    pub revision: u64,
    /// How many framebuffers are present.
    pub framebuffer_count: u64,
    /// Pointer to an array of `framebuffer_count` pointers to struct [`LimineFramebuffer`] structures.
    pub framebuffers: ArrayPtr<LimineFramebuffer>,
}

impl LimineFramebufferResponse {
    pub fn framebuffers<'a>(&'a self) -> &'a [NonNullPtr<LimineFramebuffer>] {
        self.framebuffers
            .into_slice(self.framebuffer_count as usize)
    }
}

make_struct!(
    struct LimineFramebufferRequest: [0x9d5827dcd881dd75, 0xa3148604f6fab11b] => LimineFramebufferResponse {};
);

// terminal request tag:
#[repr(C)]
#[derive(Debug)]
pub struct LimineTerminal {
    /// Number of columns provided by the terminal.
    pub cols: u64,
    /// Number of rows provided by the terminal.
    pub rows: u64,
    /// The framebuffer associated with this terminal.
    pub framebuffer: LiminePtr<LimineFramebuffer>,
}

type LimineTerminalWrite =
    Option<unsafe extern "C" fn(terminal: *const LimineTerminal, string: *const u8, length: u64)>;

#[repr(C)]
#[derive(Debug)]
pub struct LimineTerminalResponse {
    pub revision: u64,

    /// How many terminals are present.
    pub terminal_count: u64,
    /// Pointer to an array of `terminal_count` pointers to struct `limine_terminal` structures.
    pub terminals: ArrayPtr<LimineTerminal>,
    /// Physical pointer to the terminal `write()` function. The function is not thread-safe, nor
    /// reentrant, per-terminal. This means multiple terminals may be called simultaneously, and
    /// multiple callbacks may be handled simultaneously. The terminal parameter points to the
    /// [`LimineTerminal`] structure to use to output the string; the string parameter points to
    /// a string to print; the length paremeter contains the length, in bytes, of the string to print.
    write_fn: LimineTerminalWrite,
}

impl LimineTerminalResponse {
    pub fn terminals<'a>(&'a self) -> &'a [NonNullPtr<LimineTerminal>] {
        self.terminals.into_slice(self.terminal_count as usize)
    }

    pub fn write(&self) -> Option<impl Fn(&LimineTerminal, &str)> {
        let term_func = self.write_fn?;

        Some(move |terminal: &LimineTerminal, txt: &str| unsafe {
            term_func(terminal as *const _, txt.as_ptr(), txt.len() as u64);
        })
    }
}

make_struct!(
    /// Omitting this request will cause the bootloader to not initialise the terminal service.
    struct LimineTerminalRequest: [0xc8ac59310c2b0844, 0xa68d0c7265d38878] => LimineTerminalResponse {
        callback: LiminePtr<()> = LiminePtr::DEFAULT
    };
);

// 5-level paging request tag:
#[repr(C)]
#[derive(Debug)]
pub struct Limine5LevelPagingResponse {
    pub revision: u64,
}

make_struct!(
    /// The presence of this request will prompt the bootloader to turn on x86_64 5-level paging. It will not be
    /// turned on if this request is not present. If the response pointer is unchanged, 5-level paging is engaged.
    struct Limine5LevelPagingRequest: [0x94469551da9b3192, 0xebe5e86db7382888] => Limine5LevelPagingResponse {};
);

// smp request tag:
#[repr(C)]
#[derive(Debug)]
pub struct LimineSmpInfo {
    /// ACPI Processor UID as specified by the MADT.
    pub processor_id: u32,
    /// Local APIC ID of the processor as specified by the MADT.
    pub lapic_id: u32,
    pub reserved: u64,
    /// An atomic write to this field causes the parked CPU to jump to the
    /// written address, on a 64KiB (or Stack Size Request size) stack. A pointer
    /// to the struct [`LimineSmpInfo`] structure of the CPU is passed in RDI. Other
    /// than that, the CPU state will be the same as described for the bootstrap
    /// processor. This field is unused for the structure describing the bootstrap
    /// processor.
    pub goto_address: extern "C" fn(info: *const LimineSmpInfo) -> !,
    /// A free for use field.
    pub extra_argument: u64,
}

#[repr(C)]
#[derive(Debug)]
pub struct LimineSmpResponse {
    pub revision: u64,
    /// Bit 0: X2APIC has been enabled.
    pub flags: u32,
    /// The Local APIC ID of the bootstrap processor.
    pub bsp_lapic_id: u32,
    /// How many CPUs are present. It includes the bootstrap processor.
    pub cpu_count: u64,
    /// Pointer to an array of `cpu_count` pointers to struct [`LimineSmpInfo`]
    /// structures.
    pub cpus: ArrayPtr<LimineSmpInfo>,
}

impl LimineSmpResponse {
    /// Return's the SMP info array pointer as a mutable rust slice.
    ///
    /// ## Safety
    ///
    /// If this tag was returned by a bootloader mutating the slice must conform to the following
    /// rules in order to not trigger UB:
    ///
    /// - Writing to [`LimineSmpInfo::goto_address`] will cause it to start executing at the
    /// provided address.
    /// - The address pointed by [`LimineSmpInfo::goto_address`] must be that of a
    /// `extern "C" fn(&'static LimineSmpInfo) -> !`, this also means that once written this
    /// struct must not be mutated any further.
    pub fn cpus<'a>(&'a mut self) -> &'a mut [NonNullPtr<LimineSmpInfo>] {
        self.cpus.into_slice_mut(self.cpu_count as usize)
    }
}

make_struct!(
    /// The presence of this request will prompt the bootloader to bootstrap the
    /// secondary processors. This will not be done if this request is not present.
    struct LimineSmpRequest: [0x95a67b819a1b857e, 0xa0b61b723b6a73e0] => LimineSmpResponse {
        /// Bit 0: Enable X2APIC, if possible.
        flags: u32 = 0
    };
);

// memory map request tag:
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LimineMemoryMapEntryType {
    Usable = 0,
    Reserved = 1,
    AcpiReclaimable = 2,
    AcpiNvs = 3,
    BadMemory = 4,
    BootloaderReclaimable = 5,
    /// The kernel and modules loaded are not marked as usable memory. They are
    /// marked as Kernel/Modules. The entries are guaranteed to be sorted by base
    /// address, lowest to highest. Usable and bootloader reclaimable entries are
    /// guaranteed to be 4096 byte aligned for both base and length. Usable and
    /// bootloader reclaimable entries are guaranteed not to overlap with any
    /// other entry. To the contrary, all non-usable entries (including kernel/modules)
    /// are not guaranteed any alignment, nor is it guaranteed that they do not
    /// overlap other entries.
    KernelAndModules = 6,
    Framebuffer = 7,
}

#[repr(C)]
#[derive(Debug)]
pub struct LimineMemmapEntry {
    pub base: u64,
    pub len: u64,
    pub typ: LimineMemoryMapEntryType,
}

#[repr(C)]
#[derive(Debug)]
pub struct LimineMemmapResponse {
    pub revision: u64,
    /// How many memory map entries are present.
    pub entry_count: u64,
    /// Pointer to an array of `entry_count` pointers to struct [`LimineMemmapEntry`] structures.
    pub entries: ArrayPtr<LimineMemmapEntry>,
}

impl LimineMemmapResponse {
    pub fn memmap<'a>(&'a self) -> &'a [NonNullPtr<LimineMemmapEntry>] {
        self.entries.into_slice(self.entry_count as usize)
    }

    pub fn memmap_mut<'a>(&'a mut self) -> &'a mut [NonNullPtr<LimineMemmapEntry>] {
        self.entries.into_slice_mut(self.entry_count as usize)
    }
}

make_struct!(
    struct LimineMemmapRequest: [0x67cf3d9d378a806f, 0xe304acdfc50c3c62] => LimineMemmapResponse {};
);

// entry point request tag:
#[repr(C)]
#[derive(Debug)]
pub struct LimineEntryPointResponse {
    pub revision: u64,
}

make_struct!(
    struct LimineEntryPointRequest: [0x13d86c035a1cd3e1, 0x2b0caa89d8f3026a] => LimineEntryPointResponse {
        /// The requested entry point.
        entry: LiminePtr<LimineEntryPoint> = LiminePtr::DEFAULT
    };
);

// kernel file request tag:
#[repr(C)]
#[derive(Debug)]
pub struct LimineKernelFileResponse {
    pub revision: u64,
    /// Pointer to the struct [`LimineFile`] structure for the kernel file.
    pub kernel_file: LiminePtr<LimineFile>,
}

make_struct!(
    struct LimineKernelFileRequest: [0xad97e90e83f1ed67, 0x31eb5d1c5ff23b69] => LimineKernelFileResponse {};
);

// module request tag:
#[repr(C)]
#[derive(Debug)]
pub struct LimineModuleResponse {
    pub revision: u64,
    /// How many modules are present.
    pub module_count: u64,
    /// Pointer to an array of `module_count` pointers to struct [`LimineFile`] structures
    pub modules: ArrayPtr<LimineFile>,
}

impl LimineModuleResponse {
    pub fn modules<'a>(&'a self) -> &'a [NonNullPtr<LimineFile>] {
        self.modules.into_slice(self.module_count as usize)
    }
}

make_struct!(
    struct LimineModuleRequest: [0x3e7e279702be32af, 0xca1c4f3bd1280cee] => LimineModuleResponse {};
);

// RSDP request tag:
#[repr(C)]
#[derive(Debug)]
pub struct LimineRsdpResponse {
    pub revision: u64,
    /// Address of the RSDP table.
    pub address: LiminePtr<u8>,
}

make_struct!(
    struct LimineRsdpRequest: [0xc5e77b6b397e7b43, 0x27637845accdcf3c] => LimineRsdpResponse {};
);

// SMBIOS request tag:
#[repr(C)]
#[derive(Debug)]
pub struct LimineSmbiosResponse {
    pub revision: u64,
    /// Address of the 32-bit SMBIOS entry point. NULL if not present.
    pub entry_32: LiminePtr<u8>,
    /// Address of the 64-bit SMBIOS entry point. NULL if not present.
    pub entry_64: LiminePtr<u8>,
}

make_struct!(
    struct LimineSmbiosRequest: [0x9e9046f11e095391, 0xaa4a520fefbde5ee] => LimineSmbiosResponse {};
);

// EFI system table request tag:
#[repr(C)]
#[derive(Debug)]
pub struct LimineEfiSystemTableResponse {
    pub revision: u64,
    /// Address of EFI system table.
    pub address: LiminePtr<u8>,
}

make_struct!(
    struct LimineEfiSystemTableRequest: [0x5ceba5163eaaf6d6, 0x0a6981610cf65fcc] => LimineEfiSystemTableResponse {};
);

// boot time request tag:
#[repr(C)]
#[derive(Debug)]
pub struct LimineBootTimeResponse {
    pub revision: u64,
    /// The UNIX time on boot, in seconds, taken from the system RTC.
    pub boot_time: i64,
}

make_struct!(
    struct LimineBootTimeRequest: [0x502746e184c088aa, 0xfbc5ec83e6327893] => LimineBootTimeResponse {};
);

// kernel address request tag:
#[repr(C)]
#[derive(Debug)]
pub struct LimineKernelAddressResponse {
    pub revision: u64,
    /// The physical base address of the kernel.
    pub physical_base: u64,
    /// The virtual base address of the kernel.
    pub virtual_base: u64,
}

make_struct!(
    struct LimineKernelAddressRequest: [0x71ba76863cc55f63, 0xb2644a48c516a487] => LimineKernelAddressResponse {};
);

// device tree blob request tag:

/// ## Notes
/// * Information contained in the `/chosen` node may not reflect the information
/// given by bootloader tags, and as such the `/chosen` node properties should
/// be ignored.
#[repr(C)]
#[derive(Debug)]
pub struct LimineDtbResponse {
    pub revision: u64,
    /// Virtual pointer to the device tree blob.
    pub dtb_ptr: LiminePtr<u8>,
}

make_struct!(
    struct LimineDtbRequest: [0xb40ddb48fb54bac7, 0x545081493f81ffb7] => LimineDtbResponse {};
);
