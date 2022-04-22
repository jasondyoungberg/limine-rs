//! Rust crate for parsing the limine boot protocol structures.
//!
//! ## Resources
//! * [Limine Boot Protocol Specification](https://github.com/limine-bootloader/limine/blob/trunk/PROTOCOL.md)

#![no_std]

use core::fmt::Debug;

#[derive(Debug)]
#[repr(transparent)]
pub struct LimineEntryPoint(*const ());

#[repr(transparent)]
pub struct LiminePtr<T: Debug>(*const T);

impl<T: Debug> LiminePtr<T> {
    const DEFAULT: LiminePtr<T> = Self(core::ptr::null_mut() as *const T);

    /// Returns the raw pointer.
    ///
    /// # Safety
    /// The returned pointer may-be null.
    #[inline]
    unsafe fn raw_get(&self) -> *const T {
        self.0
    }

    /// Retrieve the value of the pointer. Returns an optional value since the pointer
    /// may be null.
    pub fn get(&self) -> Option<&'static T> {
        let raw_ptr = unsafe { self.raw_get() };

        if raw_ptr.is_null() {
            None
        } else {
            unsafe { Some(&*raw_ptr) }
        }
    }
}

impl LiminePtr<char> {
    /// Converts the limine string pointer into a rust string.
    pub fn to_string(&self) -> &'static str {
        let mut ptr = unsafe { self.raw_get() } as *const u8;

        // 1. Calculate the length of the string.
        let mut str_len = 0;

        // SAFTEY: We stop at the first null byte.
        unsafe {
            while *ptr != 0 {
                ptr = ptr.offset(1);
                str_len += 1;
            }
        }

        // 2. Convert the string pointer to a rust slice.
        //
        // SAFETY: We know that the string is null terminated and that the length
        // is calculated correctly.
        let slice = unsafe { core::slice::from_raw_parts(self.raw_get() as *const u8, str_len) };

        // 3. Convert the slice to a rust string.
        //
        // SAFETY: Limine strings are ensured to have valid UTF-8.
        unsafe { core::str::from_utf8_unchecked(slice) }
    }
}

impl LiminePtr<LimineEntryPoint> {
    #[inline]
    pub const fn new(entry_point: fn() -> !) -> Self {
        Self(entry_point as *const _)
    }
}

impl<T: 'static + Debug> Debug for LiminePtr<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("LiminePtr")
            .field(&format_args!("{:#x?}", unsafe { self.raw_get() }))
            .finish()
    }
}

// maker trait implementations for limine ptr
unsafe impl<T: Debug> Sync for LiminePtr<T> {}

/// Used to create the limine request struct.
macro_rules! make_struct {
    (
        $(#[$meta:meta])*
        struct $name:ident: [$id1:expr, $id2:expr] => {
            $($(#[$field_meta:meta])* $field_name:ident : $field_ty:ty = $field_default:expr),*
        };
    ) => {
        $(#[$meta])*
        #[repr(C)]
        #[derive(Debug)]
        pub struct $name {
            id: [u64; 4],
            revision: u64,

            pub $($field_name: $field_ty),*
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

                    $($field_name: $field_default),*
                }
            }

            // generate a getter method for each field:
            $($(#[$field_meta])* pub const fn $field_name(mut self, value: $field_ty) -> Self {
				self.$field_name = value;
				self
			})*
        }
    };
}

// misc structures:

#[derive(Debug)]
pub struct LimineUuid {
    pub a: u32,
    pub b: u16,
    pub c: u16,
    pub d: [u8; 8],
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
    pub path: LiminePtr<char>,
    /// A command line associated with the file.
    pub cmdline: LiminePtr<char>,
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
    pub name: LiminePtr<char>,
    /// Null-terminated string containg the version of the bootloader.
    pub version: LiminePtr<char>,
}

make_struct!(
    struct LimineBootInfoRequest: [0xf55038d8e2a1202f, 0x279426fcf5f59740] => {
        response: LiminePtr<LimineBootInfoResponse> = LiminePtr::DEFAULT
    };
);

// stack size request tag:
#[repr(C)]
#[derive(Debug)]
pub struct LimineStackSizeResponse {
    pub revision: u64,
}

make_struct!(
    struct LimineStackSizeRequest: [0x224ef0460a8e8926, 0xe1cb0fc25f46ea3d] => {
        response: LiminePtr<LimineStackSizeResponse> = LiminePtr::DEFAULT,
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
    struct LimineHhdmRequest: [0x48dcf1cb8ad2b852, 0x63984e959a98244b] => {
        response: LiminePtr<LimineHhdmResponse> = LiminePtr::DEFAULT
    };
);

// framebuffer request tag:
#[repr(C)]
#[derive(Debug)]
pub struct LimineFramebuffer {
    pub address: LiminePtr<u8>,
    pub width: u16,
    pub height: u16,
    pub pitch: u16,
    pub bpp: u16,
    pub memory_model: u8,
    pub red_mask_size: u8,
    pub red_mask_shift: u8,
    pub green_mask_size: u8,
    pub green_mask_shift: u8,
    pub blue_mask_size: u8,
    pub blue_mask_shift: u8,
    pub unused: u8,
    pub edid_size: u64,
    pub edid: LiminePtr<u8>,
}

#[repr(C)]
#[derive(Debug)]
pub struct LimineFramebufferResponse {
    pub revision: u64,
    /// How many framebuffers are present.
    pub framebuffer_count: u64,
    /// Pointer to an array of `framebuffer_count` pointers to struct [`LimineFramebuffer`] structures.
    pub framebuffers: LiminePtr<*const LimineFramebuffer>,
}

impl LimineFramebufferResponse {
    pub fn framebuffers(&self) -> Option<&'static [LimineFramebuffer]> {
        self.framebuffers.get().map(|entry| unsafe {
            core::slice::from_raw_parts(*entry, self.framebuffer_count as usize)
        })
    }
}

make_struct!(
    struct LimineFramebufferRequest: [0xcbfe81d7dd2d1977, 0x063150319ebc9b71] => {
        response: LiminePtr<LimineFramebufferResponse> = LiminePtr::DEFAULT
    };
);

// terminal request tag:
#[repr(C)]
#[derive(Debug)]
pub struct LimineTerminalResponse {
    pub revision: u64,

    pub columns: u32,
    pub rows: u32,

    write: LiminePtr<()>,
}

impl LimineTerminalResponse {
    pub fn write(&self) -> impl Fn(&str) {
        let __fn_ptr = unsafe { self.write.raw_get() };
        let __term_func =
            unsafe { core::mem::transmute::<*const (), extern "C" fn(*const i8, u64)>(__fn_ptr) };

        move |txt| {
            __term_func(txt.as_ptr() as *const i8, txt.len() as u64);
        }
    }
}

make_struct!(
    struct LimineTerminalRequest: [0x0785a0aea5d0750f, 0x1c1936fee0d6cf6e] => {
        response: LiminePtr<LimineTerminalResponse> = LiminePtr::DEFAULT,
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
    struct Limine5LevelPagingRequest: [0x94469551da9b3192, 0xebe5e86db7382888] => {
        response: LiminePtr<Limine5LevelPagingResponse> = LiminePtr::DEFAULT
    };
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
    pub goto_address: u64,
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
    pub cpus: LiminePtr<*mut LimineSmpInfo>,
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
    pub fn cpus(&mut self) -> Option<&'static mut [LimineSmpInfo]> {
        self.cpus.get().map(|entry| unsafe {
            core::slice::from_raw_parts_mut(*entry, self.cpu_count as usize)
        })
    }
}

make_struct!(
    /// The presence of this request will prompt the bootloader to bootstrap the
    /// secondary processors. This will not be done if this request is not present.
    struct LimineSmpRequest: [0x95a67b819a1b857e, 0xa0b61b723b6a73e0] => {
        response: LiminePtr<LimineSmpResponse> = LiminePtr::DEFAULT,
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
    /// Pointer to an array of `entry_count` pointers to struct [`LimineMmapEntry`] structures.
    pub entries: LiminePtr<*const LimineMemmapEntry>,
}

impl LimineMemmapResponse {
    pub fn mmap(&self) -> Option<&'static [LimineMemmapEntry]> {
        self.entries
            .get()
            .map(|entry| unsafe { core::slice::from_raw_parts(*entry, self.entry_count as usize) })
    }
}

make_struct!(
    struct LimineMmapRequest: [0x67cf3d9d378a806f, 0xe304acdfc50c3c62] => {
        response: LiminePtr<LimineMemmapResponse> = LiminePtr::DEFAULT
    };
);

// entry point request tag:
#[repr(C)]
#[derive(Debug)]
pub struct LimineEntryPointResponse {
    pub revision: u64,
}

make_struct!(
    struct LimineEntryPointRequest: [0x13d86c035a1cd3e1, 0x2b0caa89d8f3026a] => {
        response: LiminePtr<LimineEntryPointResponse> = LiminePtr::DEFAULT,
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
    struct LimineKernelFileRequest: [0xad97e90e83f1ed67, 0x31eb5d1c5ff23b69] => {
        response: LiminePtr<LimineEntryPointResponse> = LiminePtr::DEFAULT
    };
);

// module request tag:
#[repr(C)]
#[derive(Debug)]
pub struct LimineModuleResponse {
    pub revision: u64,
    /// How many modules are present.
    pub module_count: u64,
    /// Pointer to an array of `module_count` pointers to struct [`LimineFile`] structures
    pub modules: LiminePtr<*const LimineFile>,
}

impl LimineModuleResponse {
    pub fn modules(&self) -> Option<&'static [LimineFile]> {
        self.modules
            .get()
            .map(|entry| unsafe { core::slice::from_raw_parts(*entry, self.module_count as usize) })
    }
}

make_struct!(
    struct LimineModuleRequest: [0x3e7e279702be32af, 0xca1c4f3bd1280cee] => {
        response: LiminePtr<LimineModuleResponse> = LiminePtr::DEFAULT
    };
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
    struct LimineRsdpRequest: [0xc5e77b6b397e7b43, 0x27637845accdcf3c] => {
        response: LiminePtr<LimineRsdpResponse> = LiminePtr::DEFAULT
    };
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
    struct LimineSmbiosRequest: [0x9e9046f11e095391, 0xaa4a520fefbde5ee] => {
        response: LiminePtr<LimineSmbiosResponse> = LiminePtr::DEFAULT
    };
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
    struct LimineEfiSystemTableRequest: [0x5ceba5163eaaf6d6, 0x0a6981610cf65fcc] => {
        response: LiminePtr<LimineEfiSystemTableResponse> = LiminePtr::DEFAULT
    };
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
    struct LimineBootTimeRequest: [0x502746e184c088aa, 0xfbc5ec83e6327893] => {
        response: LiminePtr<LimineBootTimeResponse> = LiminePtr::DEFAULT
    };
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
    struct LimineKernelAddressRequest: [0x71ba76863cc55f63, 0xb2644a48c516a487] => {
        response: LiminePtr<LimineKernelAddressResponse> = LiminePtr::DEFAULT
    };
);
