//! Auxiliary types for the [framebuffer request](crate::request::FramebufferRequest)

use core::{ffi::c_void, ptr::NonNull};

#[derive(Clone, Copy)]
#[repr(C)]
pub(crate) struct RawFramebufferV0 {
    addr: *mut c_void,
    width: u64,
    height: u64,
    pitch: u64,
    bpp: u16,
    memory_model: MemoryModel,
    red_mask_size: u8,
    red_mask_shift: u8,
    green_mask_size: u8,
    green_mask_shift: u8,
    blue_mask_size: u8,
    blue_mask_shift: u8,
    _unused: [u8; 7],
    edid_size: u64,
    edid: Option<NonNull<u8>>,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub(crate) struct RawFramebufferV1 {
    _v0: RawFramebufferV0,
    mode_ct: u64,
    modes: *const *const VideoMode,
}

#[repr(C)]
pub(crate) union RawFramebuffer {
    v0: RawFramebufferV0,
    v1: RawFramebufferV1,
}

/// A memory model used by a framebuffer. Currently only
/// [`MemoryModel::RGB`](Self::RGB) is defined.
#[repr(transparent)]
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct MemoryModel(u8);
impl MemoryModel {
    /// This is an RGB framebuffer.
    pub const RGB: Self = Self(1);
}

/// A mode supported by the current framebuffer.
#[repr(C)]
pub struct VideoMode {
    /// The pitch (distance between rows, in bytes). This is not always the same
    /// as `(width * bpp) / 8`, as padding bytes may be added to achieve a
    /// particular alignment.
    pub pitch: u64,
    /// The width of the framebuffer, in pixels.
    pub width: u64,
    /// The height of the framebuffer, in pixels.
    pub height: u64,
    /// The number of **bits** (*not bytes*) per single pixel in the framebuffer.
    pub bpp: u16,
    /// The memory model used by this video mode. See [`MemoryModel`] for
    /// specific values.
    pub memory_model: MemoryModel,
    /// The size of the red mask, in bits. This part of the mask can be applied
    /// with `red_value & ((1 << red_mask_size) - 1)`.
    pub red_mask_size: u8,
    /// The number of bits to shift the red mask to the left. This part of the
    /// mask can be applied with `red_value << red_mask_shift`.
    pub red_mask_shift: u8,
    /// The size of the green mask, in bits. This part of the mask can be
    /// applied with `green_value & ((1 << green_mask_size) - 1)`.
    pub green_mask_size: u8,
    /// The number of bits to shift the green mask to the left. This part of the
    /// mask can be applied with `green_value << green_mask_shift`.
    pub green_mask_shift: u8,
    /// The size of the blue mask, in bits. This part of the mask can be applied
    /// with `blue_value & ((1 << blue_mask_size) - 1)`.
    pub blue_mask_size: u8,
    /// The number of bits to shift the blue mask to the left. This part of the
    /// mask can be applied with `blue_value << blue_mask_shift`.
    pub blue_mask_shift: u8,
}

/// A pointer to a framebuffer.
///
/// # Why is this a wrapper type?
/// Two revisions currently exist of the framebuffer type. However, the type
/// itself has no revision field. In order to keep this type safe, we wrap the
/// pointer with its associated revision taken from the response.
pub struct Framebuffer<'a> {
    revision: u64,
    inner: &'a RawFramebuffer,
}
impl<'a> Framebuffer<'a> {
    pub(crate) fn new(revision: u64, inner: &'a RawFramebuffer) -> Self {
        Self { revision, inner }
    }

    /// The address of the framebuffer. Note that no synchronization is done on
    /// this pointer, so you yourself must synchronize all access. In addition,
    /// the pointer may point to uninitialized bytes at boot, so dereferencing
    /// it will cause UB.
    pub fn addr(&self) -> *mut u8 {
        unsafe { self.inner.v0 }.addr.cast()
    }

    /// The width of the framebuffer in its current mode.
    pub fn width(&self) -> u64 {
        unsafe { self.inner.v0 }.width
    }
    /// The height of the framebuffer in its current mode.
    pub fn height(&self) -> u64 {
        unsafe { self.inner.v0 }.height
    }

    /// The pitch (distance between rows, in bytes) of the framebuffer in the
    /// current mode. This is not always the same as `(width * bpp) / 8`, as
    /// padding bytes may be added to achieve a particular alignment.
    pub fn pitch(&self) -> u64 {
        unsafe { self.inner.v0 }.pitch
    }
    /// The number of **bits** (*not bytes*) per pixel of the framebuffer in the
    /// current mode.
    pub fn bpp(&self) -> u16 {
        unsafe { self.inner.v0 }.bpp
    }

    /// The memory model of the framebuffer in the current mode. See
    /// [`MemoryModel`] for specific values.
    pub fn memory_model(&self) -> MemoryModel {
        unsafe { self.inner.v0 }.memory_model
    }

    /// The size of the red mask, in bits. This part of the mask can be applied
    /// with `red_value & ((1 << red_mask_size) - 1)`.
    pub fn red_mask_size(&self) -> u8 {
        unsafe { self.inner.v0 }.red_mask_size
    }
    /// The number of bits to shift the red mask to the left. This part of the
    /// mask can be applied with `red_value << red_mask_shift`.
    pub fn red_mask_shift(&self) -> u8 {
        unsafe { self.inner.v0 }.red_mask_shift
    }
    /// The size of the green mask, in bits. This part of the mask can be
    /// applied with `green_value & ((1 << green_mask_size) - 1)`.
    pub fn green_mask_size(&self) -> u8 {
        unsafe { self.inner.v0 }.green_mask_size
    }
    /// The number of bits to shift the green mask to the left. This part of the
    /// mask can be applied with `green_value << green_mask_shift`.
    pub fn green_mask_shift(&self) -> u8 {
        unsafe { self.inner.v0 }.green_mask_shift
    }
    /// The size of the blue mask, in bits. This part of the mask can be applied
    /// with `blue_value & ((1 << blue_mask_size) - 1)`.
    pub fn blue_mask_size(&self) -> u8 {
        unsafe { self.inner.v0 }.blue_mask_size
    }
    /// The number of bits to shift the blue mask to the left. This part of the
    /// mask can be applied with `blue_value << blue_mask_shift`.
    pub fn blue_mask_shift(&self) -> u8 {
        unsafe { self.inner.v0 }.blue_mask_shift
    }

    /// The raw EDID bytes of the display attached to this framebuffer.
    pub fn edid(&self) -> Option<&[u8]> {
        unsafe {
            self.inner.v0.edid.map(|ptr| {
                core::slice::from_raw_parts(ptr.as_ptr(), self.inner.v0.edid_size as usize)
            })
        }
    }

    /// The video modes supported on this framebuffer. Only available on
    /// revision 1 and above.
    pub fn modes(&self) -> Option<&[&VideoMode]> {
        match self.revision {
            0 => None,
            1.. => Some(unsafe {
                core::slice::from_raw_parts(
                    self.inner.v1.modes.cast(),
                    self.inner.v1.mode_ct as usize,
                )
            }),
        }
    }
}
