//! Auxiliary types for the [module request](crate::request::ModuleRequest)

use core::ffi::{c_char, CStr};

use bitflags::bitflags;

bitflags! {
    /// Flags for internal modules
    #[derive(PartialEq, Eq, Clone, Copy)]
    pub struct ModuleFlags: u64 {
        /// The module is required. If it is not found, the bootloader will
        /// refuse to boot.
        const REQUIRED = 1 << 0;
        /// The module is GZ-compressed and will be uncompressed by the
        /// bootloader. This is only honored on response revision 2 and greater.
        const COMPRESSED = 1 << 1;
    }
}

/// Create a NUL-terminated C string from a string literal
#[macro_export]
macro_rules! cstr {
    () => {
        unsafe { core::ffi::CStr::from_bytes_with_nul_unchecked(b"\0") }
    };
    ($s:expr) => {
        unsafe { core::ffi::CStr::from_bytes_with_nul_unchecked(concat!($s, "\0").as_bytes()) }
    };
}

/// An internal module that the kernel requests from the bootloader. Only
/// available with request revision 1 and greater.
#[repr(C)]
pub struct InternalModule {
    path: *const c_char,
    cmdline: *const c_char,
    flags: ModuleFlags,
}
unsafe impl Sync for InternalModule {}
unsafe impl Send for InternalModule {}
impl InternalModule {
    /// Create a new internal module with no path, no cmdline and no flags.
    pub const fn new() -> Self {
        Self {
            path: b"\0".as_ptr().cast(),
            cmdline: b"\0".as_ptr().cast(),
            flags: ModuleFlags::empty(),
        }
    }

    /// Set the path of the internal module. This function returns the new value.
    ///
    /// # Parameters
    /// - `path`: The new value of the field.
    pub const fn with_path(mut self, path: &'static CStr) -> Self {
        self.path = path.as_ptr();
        self
    }
    /// Set the path of the internal module. This function operates in place.
    ///
    /// # Parameters
    /// - `path`: The new value of the field.
    pub fn set_path(&mut self, path: &'static CStr) {
        self.path = path.as_ptr();
    }

    /// Set the command-line for the module. This function returns the new value.
    ///
    /// # Parameters
    /// - `cmdline`: The new value of the field.
    pub const fn with_cmdline(mut self, cmdline: &'static CStr) -> Self {
        self.cmdline = cmdline.as_ptr();
        self
    }
    /// Set the command-line for the module. This function operates in place.
    ///
    /// # Parameters
    /// - `cmdline`: The new value of the field.
    pub fn set_cmdline(&mut self, cmdline: &'static CStr) {
        self.cmdline = cmdline.as_ptr();
    }

    /// Set the flags for the module. This function returns the new value.
    ///
    /// # Parameters
    /// - `flags`: The new value of the field.
    pub const fn with_flags(mut self, flags: ModuleFlags) -> Self {
        self.flags = flags;
        self
    }
    /// Set the flags for the module. This function operates in place.
    ///
    /// # Parameters
    /// - `flags`: The new value of the field.
    pub fn set_flags(&mut self, flags: ModuleFlags) {
        self.flags = flags;
    }

    /// Returns the module's path as a byte slice with unspecified encoding.
    pub fn path(&self) -> &[u8] {
        unsafe { CStr::from_ptr(self.path).to_bytes() }
    }
    /// Returns the module's command-line as a byte slice with unspecified
    /// encoding.
    pub fn cmdline(&self) -> &[u8] {
        unsafe { CStr::from_ptr(self.cmdline).to_bytes() }
    }
    /// Returns the module's flags.
    pub fn flags(&self) -> ModuleFlags {
        self.flags
    }
}
