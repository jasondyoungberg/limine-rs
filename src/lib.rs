//! Rust crate for parsing the limine boot protocol structures.
//!
//! ## Resources
//! * [Limine Boot Protocol Specification](https://github.com/limine-bootloader/limine/blob/trunk/PROTOCOL.md)

#![no_std]
#![feature(const_nonnull_new)]
#![allow(deprecated)]

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
pub struct EntryPoint(*const ());

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

// SAFETY: The underlying type (`T`) implements {Send, Sync} so, it is safe
//         for NonNullPtr<T> to implement {Send, Sync}.
unsafe impl<T: Send> Send for NonNullPtr<T> {}
unsafe impl<T: Sync> Sync for NonNullPtr<T> {}

#[repr(transparent)]
pub struct Ptr<T> {
    ptr: Option<NonNull<T>>,
    // This marker does not affect the variance but is required for
    // dropck to undestand that we logically own a `T`.
    //
    // TODO: Use `Unique<T>` when stabalized!
    _phantom: PhantomData<T>,
}

impl<T> Ptr<T> {
    const DEFAULT: Ptr<T> = Self {
        ptr: None,
        _phantom: PhantomData,
    };

    #[inline]
    pub fn as_ptr(&self) -> Option<*mut T> {
        Some(self.ptr?.as_ptr())
    }

    #[inline]
    pub fn get<'a>(&self) -> Option<&'a T> {
        // SAFETY: According to the specication the bootloader provides a aligned
        //         pointer and there is no public API to construct a [`Ptr`]
        //         so, its safe to assume that the [`NonNull::as_ref`] are applied. 
        //         If not, its the bootloader's fault that they have violated the
        //         specification!.
        //
        // Also, we have a shared reference to the data and there is no
        // legal way to mutate it, unless through [`Ptr::as_ptr`]
        // (requires pointer dereferencing which is unsafe) or [`Ptr::get_mut`]
        // (requires exclusive access to the [`Ptr`]).
        self.ptr.map(|e| unsafe { e.as_ref() })
    }

    #[inline]
    pub fn get_mut<'a>(&mut self) -> Option<&'a mut T> {
        // SAFETY: Check the safety for [`Ptr::get`] and we have
        // exclusive access to the data.
        self.ptr.as_mut().map(|e| unsafe { e.as_mut() })
    }
}

impl Ptr<c_char> {
    /// Converts the  string pointer into a rust string.
    pub fn to_str(&self) -> Option<&CStr> {
        // SAFETY: According to the  specification, the pointer points
        //         to a valid C string with a NULL terminator of size less than
        //         `isize::MAX`. Also we know that the `Ptr` is a valid C
        //         string, because it has a `T` of `c_char`. See the [`Ptr::get`] 
        //         for more details.
        unsafe { Some(CStr::from_ptr(self.as_ptr()?)) }
    }
}

impl Ptr<EntryPoint> {
    #[inline]
    pub const fn new(entry_point: fn() -> !) -> Self {
        Self {
            ptr: NonNull::new(entry_point as *mut _),
            _phantom: PhantomData,
        }
    }
}

impl<T: Debug> Debug for Ptr<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("Ptr")
            .field(&format_args!("{:#x?}", self.get()))
            .finish()
    }
}

// SAFETY: The underlying type (`T`) implements {Send, Sync} so, it is safe
//         for Ptr<T> to implement {Send, Sync}.
unsafe impl<T: Send> Send for Ptr<T> {}
unsafe impl<T: Sync> Sync for Ptr<T> {}

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

/// Used to create the  request struct.
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

            // XXX: The response is required to be wrapped inside an unsafe cell, since
            //      by default the response is set to NULL and when the compiler does not see
            //      any writes to the field, it is free to assume that the response is NULL. In
            //      our situation the bootloader mutates the field and we need to ensure that
            //      the compiler does not optimize the read away.
            response: UnsafeCell<Ptr<$response_ty>>,
            $(pub $field_name: $field_ty),*
        }

        impl $name {
            // XXX: The request ID is composed of 4 64-bit wide unsigned integers but the first
            //      two remain constant. This is refered as `_COMMON_MAGIC` in the  protocol
            //      header.
            pub const ID: [u64; 4] = [0xc7b1dd30df4c8b88, 0x0a82e883a194f07b, $id1, $id2];

            pub const fn new(revision: u64) -> Self {
                Self {
                    id: Self::ID,
                    revision,

                    response: UnsafeCell::new(Ptr::DEFAULT),
                    $($field_name: $field_default),*
                }
            }

            pub fn get_response(&self) -> Ptr<$response_ty> {
                unsafe { core::ptr::read_volatile(self.response.get()) }
            }

            // generate a getter method for each field:
            $($(#[$field_meta])* pub const fn $field_name(mut self, value: $field_ty) -> Self {
                self.$field_name = value;
                self
            })*
        }

        // maker trait implementations for  request struct:
        unsafe impl Sync for $name {}
    };
}

// misc structures:

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Uuid {
    pub a: u32,
    pub b: u16,
    pub c: u16,
    pub d: [u8; 8],
}

#[cfg(feature = "into-uuid")]
impl From<Uuid> for uuid::Uuid {
    fn from(lim: Uuid) -> Self {
        Self::from_fields(lim.a, lim.b, lim.c, &lim.d)
    }
}

#[cfg(feature = "into-uuid")]
impl From<uuid::Uuid> for Uuid {
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

#[repr(C)]
#[derive(Debug)]
pub struct File {
    /// Revision of this structure.
    pub revision: u64,
    /// The address of the file.
    pub base: Ptr<u8>,
    /// The size of the file.
    pub length: u64,
    /// The path of the file within the volume, with a leading slash.
    pub path: Ptr<c_char>,
    /// A command line associated with the file.
    pub cmdline: Ptr<c_char>,
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
    pub gpt_disk_uuid: Uuid,
    /// If non-0, this is the UUID of the partition the file was loaded from as reported in the GPT.
    pub gpt_part_uuid: Uuid,
    /// If non-0, this is the UUID of the filesystem of the partition the file was loaded from.
    pub part_uuid: Uuid,
}

// boot info request tag:
#[repr(C)]
#[derive(Debug)]
pub struct BootInfoResponse {
    pub revision: u64,
    /// Null-terminated string containing the name of the bootloader.
    pub name: Ptr<c_char>,
    /// Null-terminated string containg the version of the bootloader.
    pub version: Ptr<c_char>,
}

make_struct!(
    struct BootInfoRequest: [0xf55038d8e2a1202f, 0x279426fcf5f59740] => BootInfoResponse {};
);

// stack size request tag:
#[repr(C)]
#[derive(Debug)]
pub struct StackSizeResponse {
    pub revision: u64,
}

make_struct!(
    struct StackSizeRequest: [0x224ef0460a8e8926, 0xe1cb0fc25f46ea3d] => StackSizeResponse {
        /// The requested stack size (also used for SMP processors).
        stack_size: u64 = 0
    };
);

// HHDM request tag:
#[repr(C)]
#[derive(Debug)]
pub struct HhdmResponse {
    pub revision: u64,
    /// The virtual address offset of the beginning of the higher half direct map.
    pub offset: u64,
}

make_struct!(
    struct HhdmRequest: [0x48dcf1cb8ad2b852, 0x63984e959a98244b] => HhdmResponse {};
);

// framebuffer request tag:
#[repr(C)]
#[derive(Debug)]
pub struct Framebuffer {
    pub address: Ptr<u8>,
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
    pub edid: Ptr<u8>,
}

impl Framebuffer {
    /// Returns the size of the framebuffer.
    pub fn size(&self) -> usize {
        self.pitch as usize * self.height as usize * (self.bpp as usize / 8)
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct FramebufferResponse {
    pub revision: u64,
    /// How many framebuffers are present.
    pub framebuffer_count: u64,
    /// Pointer to an array of `framebuffer_count` pointers to struct [`Framebuffer`] structures.
    pub framebuffers: ArrayPtr<Framebuffer>,
}

impl FramebufferResponse {
    pub fn framebuffers<'a>(&'a self) -> &'a [NonNullPtr<Framebuffer>] {
        self.framebuffers
            .into_slice(self.framebuffer_count as usize)
    }
}

make_struct!(
    struct FramebufferRequest: [0x9d5827dcd881dd75, 0xa3148604f6fab11b] => FramebufferResponse {};
);

// terminal request tag:
#[repr(C)]
#[derive(Debug)]
#[deprecated(note = "This feature is deprecated, do not use if possible.")]
pub struct Terminal {
    /// Number of columns provided by the terminal.
    pub cols: u64,
    /// Number of rows provided by the terminal.
    pub rows: u64,
    /// The framebuffer associated with this terminal.
    pub framebuffer: Ptr<Framebuffer>,
}

#[deprecated(note = "This feature is deprecated, do not use if possible.")]
type TerminalWrite =
    Option<unsafe extern "C" fn(terminal: *const Terminal, string: *const u8, length: u64)>;

#[repr(C)]
#[derive(Debug)]
#[deprecated(note = "This feature is deprecated, do not use if possible.")]
pub struct TerminalResponse {
    pub revision: u64,

    /// How many terminals are present.
    pub terminal_count: u64,
    /// Pointer to an array of `terminal_count` pointers to struct `_terminal` structures.
    pub terminals: ArrayPtr<Terminal>,
    /// Physical pointer to the terminal `write()` function. The function is not thread-safe, nor
    /// reentrant, per-terminal. This means multiple terminals may be called simultaneously, and
    /// multiple callbacks may be handled simultaneously. The terminal parameter points to the
    /// [`Terminal`] structure to use to output the string; the string parameter points to
    /// a string to print; the length paremeter contains the length, in bytes, of the string to print.
    write_fn: TerminalWrite,
}

#[deprecated(note = "This feature is deprecated, do not use if possible.")]
impl TerminalResponse {
    pub fn terminals<'a>(&'a self) -> &'a [NonNullPtr<Terminal>] {
        self.terminals.into_slice(self.terminal_count as usize)
    }

    pub fn write(&self) -> Option<impl Fn(&Terminal, &str)> {
        let term_func = self.write_fn?;

        Some(move |terminal: &Terminal, txt: &str| unsafe {
            term_func(terminal as *const _, txt.as_ptr(), txt.len() as u64);
        })
    }
}

make_struct!(
    /// Omitting this request will cause the bootloader to not initialise the terminal service.
    #[deprecated(note = "This feature is deprecated, do not use if possible.")]
    struct TerminalRequest: [0xc8ac59310c2b0844, 0xa68d0c7265d38878] => TerminalResponse {
        callback: Ptr<()> = Ptr::DEFAULT
    };
);

// 5-level paging request tag:
#[repr(C)]
#[derive(Debug)]
pub struct Level5PagingResponse {
    pub revision: u64,
}

make_struct!(
    /// The presence of this request will prompt the bootloader to turn on x86_64 5-level paging. It will not be
    /// turned on if this request is not present. If the response pointer is unchanged, 5-level paging is engaged.
    struct Level5PagingRequest: [0x94469551da9b3192, 0xebe5e86db7382888] => Level5PagingResponse {};
);

// smp request tag:
#[repr(C)]
#[derive(Debug)]
pub struct SmpInfo {
    /// ACPI Processor UID as specified by the MADT.
    pub processor_id: u32,
    /// Local APIC ID of the processor as specified by the MADT.
    pub lapic_id: u32,
    pub reserved: u64,
    /// An atomic write to this field causes the parked CPU to jump to the
    /// written address, on a 64KiB (or Stack Size Request size) stack. A pointer
    /// to the struct [`SmpInfo`] structure of the CPU is passed in RDI. Other
    /// than that, the CPU state will be the same as described for the bootstrap
    /// processor. This field is unused for the structure describing the bootstrap
    /// processor.
    pub goto_address: extern "C" fn(info: *const SmpInfo) -> !,
    /// A free for use field.
    pub extra_argument: u64,
}

#[repr(C)]
#[derive(Debug)]
pub struct SmpResponse {
    pub revision: u64,
    /// Bit 0: X2APIC has been enabled.
    pub flags: u32,
    /// The Local APIC ID of the bootstrap processor.
    pub bsp_lapic_id: u32,
    /// How many CPUs are present. It includes the bootstrap processor.
    pub cpu_count: u64,
    /// Pointer to an array of `cpu_count` pointers to struct [`SmpInfo`]
    /// structures.
    pub cpus: ArrayPtr<SmpInfo>,
}

impl SmpResponse {
    /// Return's the SMP info array pointer as a mutable rust slice.
    ///
    /// ## Safety
    ///
    /// If this tag was returned by a bootloader mutating the slice must conform to the following
    /// rules in order to not trigger UB:
    ///
    /// - Writing to [`SmpInfo::goto_address`] will cause it to start executing at the
    /// provided address.
    /// - The address pointed by [`SmpInfo::goto_address`] must be that of a
    /// `extern "C" fn(&'static SmpInfo) -> !`, this also means that once written this
    /// struct must not be mutated any further.
    pub fn cpus<'a>(&'a mut self) -> &'a mut [NonNullPtr<SmpInfo>] {
        self.cpus.into_slice_mut(self.cpu_count as usize)
    }
}

make_struct!(
    /// The presence of this request will prompt the bootloader to bootstrap the
    /// secondary processors. This will not be done if this request is not present.
    struct SmpRequest: [0x95a67b819a1b857e, 0xa0b61b723b6a73e0] => SmpResponse {
        /// Bit 0: Enable X2APIC, if possible.
        flags: u32 = 0
    };
);

// memory map request tag:
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MemoryMapEntryType {
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
pub struct MemmapEntry {
    pub base: u64,
    pub len: u64,
    pub typ: MemoryMapEntryType,
}

#[repr(C)]
#[derive(Debug)]
pub struct MemmapResponse {
    pub revision: u64,
    /// How many memory map entries are present.
    pub entry_count: u64,
    /// Pointer to an array of `entry_count` pointers to struct [`MemmapEntry`] structures.
    pub entries: ArrayPtr<MemmapEntry>,
}

impl MemmapResponse {
    pub fn memmap<'a>(&'a self) -> &'a [NonNullPtr<MemmapEntry>] {
        self.entries.into_slice(self.entry_count as usize)
    }

    pub fn memmap_mut<'a>(&'a mut self) -> &'a mut [NonNullPtr<MemmapEntry>] {
        self.entries.into_slice_mut(self.entry_count as usize)
    }
}

make_struct!(
    struct MemmapRequest: [0x67cf3d9d378a806f, 0xe304acdfc50c3c62] => MemmapResponse {};
);

// entry point request tag:
#[repr(C)]
#[derive(Debug)]
pub struct EntryPointResponse {
    pub revision: u64,
}

make_struct!(
    struct EntryPointRequest: [0x13d86c035a1cd3e1, 0x2b0caa89d8f3026a] => EntryPointResponse {
        /// The requested entry point.
        entry: Ptr<EntryPoint> = Ptr::DEFAULT
    };
);

// kernel file request tag:
#[repr(C)]
#[derive(Debug)]
pub struct KernelFileResponse {
    pub revision: u64,
    /// Pointer to the struct [`File`] structure for the kernel file.
    pub kernel_file: Ptr<File>,
}

make_struct!(
    struct KernelFileRequest: [0xad97e90e83f1ed67, 0x31eb5d1c5ff23b69] => KernelFileResponse {};
);

// module request tag:
#[repr(C)]
#[derive(Debug)]
pub struct ModuleResponse {
    pub revision: u64,
    /// How many modules are present.
    pub module_count: u64,
    /// Pointer to an array of `module_count` pointers to struct [`File`] structures
    pub modules: ArrayPtr<File>,
}

impl ModuleResponse {
    pub fn modules<'a>(&'a self) -> &'a [NonNullPtr<File>] {
        self.modules.into_slice(self.module_count as usize)
    }
}

make_struct!(
    struct ModuleRequest: [0x3e7e279702be32af, 0xca1c4f3bd1280cee] => ModuleResponse {};
);

// RSDP request tag:
#[repr(C)]
#[derive(Debug)]
pub struct RsdpResponse {
    pub revision: u64,
    /// Address of the RSDP table.
    pub address: Ptr<u8>,
}

make_struct!(
    struct RsdpRequest: [0xc5e77b6b397e7b43, 0x27637845accdcf3c] => RsdpResponse {};
);

// SMBIOS request tag:
#[repr(C)]
#[derive(Debug)]
pub struct SmbiosResponse {
    pub revision: u64,
    /// Address of the 32-bit SMBIOS entry point. NULL if not present.
    pub entry_32: Ptr<u8>,
    /// Address of the 64-bit SMBIOS entry point. NULL if not present.
    pub entry_64: Ptr<u8>,
}

make_struct!(
    struct SmbiosRequest: [0x9e9046f11e095391, 0xaa4a520fefbde5ee] => SmbiosResponse {};
);

// EFI system table request tag:
#[repr(C)]
#[derive(Debug)]
pub struct EfiSystemTableResponse {
    pub revision: u64,
    /// Address of EFI system table.
    pub address: Ptr<u8>,
}

make_struct!(
    struct EfiSystemTableRequest: [0x5ceba5163eaaf6d6, 0x0a6981610cf65fcc] => EfiSystemTableResponse {};
);

// boot time request tag:
#[repr(C)]
#[derive(Debug)]
pub struct BootTimeResponse {
    pub revision: u64,
    /// The UNIX time on boot, in seconds, taken from the system RTC.
    pub boot_time: i64,
}

make_struct!(
    struct BootTimeRequest: [0x502746e184c088aa, 0xfbc5ec83e6327893] => BootTimeResponse {};
);

// kernel address request tag:
#[repr(C)]
#[derive(Debug)]
pub struct KernelAddressResponse {
    pub revision: u64,
    /// The physical base address of the kernel.
    pub physical_base: u64,
    /// The virtual base address of the kernel.
    pub virtual_base: u64,
}

make_struct!(
    struct KernelAddressRequest: [0x71ba76863cc55f63, 0xb2644a48c516a487] => KernelAddressResponse {};
);

// device tree blob request tag:

/// ## Notes
/// * Information contained in the `/chosen` node may not reflect the information
/// given by bootloader tags, and as such the `/chosen` node properties should
/// be ignored.
#[repr(C)]
#[derive(Debug)]
pub struct DtbResponse {
    pub revision: u64,
    /// Virtual pointer to the device tree blob.
    pub dtb_ptr: Ptr<u8>,
}

make_struct!(
    struct DtbRequest: [0xb40ddb48fb54bac7, 0x545081493f81ffb7] => DtbResponse {};
);
