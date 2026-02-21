// Copyright © 2026, __robot@PLT
// SPDX-License-Identifier: MIT

use core::ffi::{CStr, c_char};

pub const INTERNAL_MODULE_REQUIRED: u64 = 1 << 0;
pub const INTERNAL_MODULE_COMPRESSED: u64 = 1 << 1;

pub struct InternalModule {
    path: *const c_char,
    string: *const c_char,
    pub flags: u64,
}

impl InternalModule {
    pub const fn new(path: &'static CStr, string: &'static CStr, flags: u64) -> Self {
        Self {
            path: path.as_ptr(),
            string: string.as_ptr(),
            flags,
        }
    }

    pub fn path(&self) -> &CStr {
        unsafe { CStr::from_ptr(self.path) }
    }

    pub fn string(&self) -> &CStr {
        unsafe { CStr::from_ptr(self.string) }
    }
}
