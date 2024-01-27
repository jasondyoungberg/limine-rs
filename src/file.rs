//! File structure and related functions.

use core::{
    ffi::{c_char, c_void, CStr},
    mem::MaybeUninit,
    num::NonZeroU32,
};

#[cfg(feature = "ipaddr")]
use core::net::{Ipv4Addr, SocketAddrV4};

/// A UUID. With the `uuid` feature, this can be converted directly to
/// [`uuid::Uuid`] via [`Into`], and the reverse via [`From`].
#[repr(C)]
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Uuid {
    /// The first 32 bits of the UUID.
    pub a: u32,
    /// The next 16 bits of the UUID.
    pub b: u16,
    /// The next 16 bits of the UUID.
    pub c: u16,
    /// The last 64 bits of the UUID.
    pub d: [u8; 8],
}
impl Uuid {
    fn non_zero(&self) -> Option<Self> {
        (self.a != 0 || self.b != 0 || self.c != 0 || self.d != [0; 8]).then_some(*self)
    }
}
#[cfg(feature = "uuid")]
impl From<uuid::Uuid> for Uuid {
    fn from(uuid: uuid::Uuid) -> Self {
        Self {
            a: uuid.as_fields().0,
            b: uuid.as_fields().1,
            c: uuid.as_fields().2,
            d: *uuid.as_fields().3,
        }
    }
}
#[cfg(feature = "uuid")]
impl From<Uuid> for uuid::Uuid {
    fn from(uuid: Uuid) -> Self {
        Self::from_fields(uuid.a, uuid.b, uuid.c, &uuid.d)
    }
}

/// A media type for a file.
#[derive(PartialEq, Eq, Clone, Copy)]
#[repr(transparent)]
pub struct MediaType(u32);
impl MediaType {
    /// Unknown media type.
    pub const GENERIC: Self = Self(0);
    /// A CD-ROM.
    pub const OPTICAL: Self = Self(1);
    /// A TFTP server.
    pub const TFTP: Self = Self(2);
}

/// A file loaded by the bootloader. Returned from
/// [`KernelFileRequest`](crate::request::KernelFileRequest) and
/// [`ModuleRequest`](crate::request::ModuleRequest).
#[repr(C)]
pub struct File {
    revision: u64,
    addr: *mut c_void,
    size: u64,
    path: *const c_char,
    cmdline: *const c_char,
    media_type: MediaType,
    _unused: MaybeUninit<u32>,
    tftp_ip: Option<NonZeroU32>,
    tftp_port: Option<NonZeroU32>,
    partition_idx: Option<NonZeroU32>,
    mbr_disk_id: Option<NonZeroU32>,
    gpt_disk_id: Uuid,
    gpt_partition_id: Uuid,
    partition_uuid: Uuid,
}
impl File {
    /// Get the revision of the file. Currently, this is always 0.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// The base address of the file. Note that this is not necessarily a
    /// pointer to executable code. It simply points to the raw file.
    pub fn addr(&self) -> *mut u8 {
        self.addr.cast()
    }
    /// The size of the file, in bytes.
    pub fn size(&self) -> u64 {
        self.size
    }

    /// The path of the file. This is the path that was passed to the bootloader
    /// in either the configuration file or the `internal_modules` field of the
    /// [`ModuleRequest`](crate::request::ModuleRequest).
    ///
    /// It is returned as a raw byte slice, and the encoding is unspecified.
    pub fn path(&self) -> &[u8] {
        let c_str = unsafe { CStr::from_ptr(self.path) };
        c_str.to_bytes()
    }
    /// The command line of the file. This is the command line that was passed
    /// to the bootloader in either the configuration file or the
    /// `internal_modules` field of the
    /// [`ModuleRequest`](crate::request::ModuleRequest).
    ///
    /// It is returned as a raw byte slice, and the encoding is unspecified.
    pub fn cmdline(&self) -> &[u8] {
        let c_str = unsafe { CStr::from_ptr(self.cmdline) };
        c_str.to_bytes()
    }

    /// The media type of the file. See [`MediaType`] for more information.
    pub fn media_type(&self) -> MediaType {
        self.media_type
    }

    /// The IP address of the TFTP server, if the file was loaded from a TFTP.
    #[cfg(feature = "ipaddr")]
    pub fn tftp_ip(&self) -> Option<Ipv4Addr> {
        self.tftp_ip.map(|v| {
            let bytes = v.get().to_ne_bytes();
            Ipv4Addr::new(bytes[0], bytes[1], bytes[2], bytes[3])
        })
    }
    /// The IP address of the TFTP server, if the file was loaded from a TFTP.
    #[cfg(not(feature = "ipaddr"))]
    pub fn tftp_ip(&self) -> Option<NonZeroU32> {
        self.tftp_ip
    }
    /// The port of the TFTP server, if the file was loaded from a TFTP.
    pub fn tftp_port(&self) -> Option<NonZeroU32> {
        self.tftp_port
    }
    /// The address of the TFTP server, if the file was loaded from a TFTP. This
    /// is simply a combination of [`tftp_ip`](Self::tftp_ip) and
    /// [`tftp_port`](Self::tftp_port).
    #[cfg(feature = "ipaddr")]
    pub fn tftp_addr(&self) -> Option<SocketAddrV4> {
        self.tftp_ip()
            .and_then(|ip| Some(SocketAddrV4::new(ip, self.tftp_port()?.get() as u16)))
    }

    /// The partition index of the file, if the file was loaded from a partition.
    pub fn partition_idx(&self) -> Option<NonZeroU32> {
        self.partition_idx
    }
    /// The MBR disk ID of the file, if the file was loaded from an MBR disk.
    pub fn mbr_disk_id(&self) -> Option<NonZeroU32> {
        self.mbr_disk_id
    }

    /// The GPT disk UUID of the file, if the file was loaded from a GPT disk.
    pub fn gpt_disk_id(&self) -> Option<Uuid> {
        self.gpt_disk_id.non_zero()
    }
    /// The GPT partition UUID of the file, if the file was loaded from a GPT
    /// partition.
    pub fn gpt_partition_id(&self) -> Option<Uuid> {
        self.gpt_partition_id.non_zero()
    }
    /// The partition UUID of the file, if the file was loaded from a partition
    /// with a UUID.
    pub fn partition_uuid(&self) -> Option<Uuid> {
        self.partition_uuid.non_zero()
    }
}
