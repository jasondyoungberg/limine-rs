#![deprecated(since = "0.4.0", note = "please use `limine::mp` instead")]

//! Auxiliary types for the [SMP request](crate::request::SmpRequest).

use crate::mp;

#[deprecated(since = "0.4.0", note = "please use `limine::mp::GotoAddress` instead")]
/// A function pointer that the core will jump to when it is written to.
pub type GotoAddress = mp::GotoAddress;

#[deprecated(since = "0.4.0", note = "please use `limine::mp::Cpu` instead")]
/// A CPU entry in the SMP request.
pub type Cpu = mp::Cpu;

#[deprecated(
    since = "0.4.0",
    note = "please use `limine::mp::RequestFlags` instead"
)]
/// Flags for the [SMP request](crate::request::SmpRequest).
pub type RequestFlags = mp::RequestFlags;

#[deprecated(
    since = "0.4.0",
    note = "please use `limine::mp::ResponseFlags` instead"
)]
/// Flags for the [SMP response](crate::response::SmpResponse).
pub type ResponseFlags = mp::ResponseFlags;
