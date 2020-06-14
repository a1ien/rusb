use std::{fmt, result};

use libusb1_sys::constants::*;

/// A result of a function that may return a `Error`.
pub type Result<T> = result::Result<T, Error>;

/// Errors returned by the `libusb` library.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Error {
    /// Input/output error.
    Io,

    /// Invalid parameter.
    InvalidParam,

    /// Access denied (insufficient permissions).
    Access,

    /// No such device (it may have been disconnected).
    NoDevice,

    /// Entity not found.
    NotFound,

    /// Resource busy.
    Busy,

    /// Operation timed out.
    Timeout,

    /// Overflow.
    Overflow,

    /// Pipe error.
    Pipe,

    /// System call interrupted (perhaps due to signal).
    Interrupted,

    /// Insufficient memory.
    NoMem,

    /// Operation not supported or unimplemented on this platform.
    NotSupported,

    /// The device returned a malformed descriptor.
    BadDescriptor,

    /// Other error.
    Other,
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        fmt.write_str(match self {
            Error::Io => "Input/Output Error",
            Error::InvalidParam => "Invalid parameter",
            Error::Access => "Access denied (insufficient permissions)",
            Error::NoDevice => "No such device (it may have been disconnected)",
            Error::NotFound => "Entity not found",
            Error::Busy => "Resource busy",
            Error::Timeout => "Operation timed out",
            Error::Overflow => "Overflow",
            Error::Pipe => "Pipe error",
            Error::Interrupted => "System call interrupted (perhaps due to signal)",
            Error::NoMem => "Insufficient memory",
            Error::NotSupported => "Operation not supported or unimplemented on this platform",
            Error::BadDescriptor => "Malformed descriptor",
            Error::Other => "Other error",
        })
    }
}

impl std::error::Error for Error {}

#[doc(hidden)]
pub(crate) fn from_libusb(err: i32) -> Error {
    match err {
        LIBUSB_ERROR_IO => Error::Io,
        LIBUSB_ERROR_INVALID_PARAM => Error::InvalidParam,
        LIBUSB_ERROR_ACCESS => Error::Access,
        LIBUSB_ERROR_NO_DEVICE => Error::NoDevice,
        LIBUSB_ERROR_NOT_FOUND => Error::NotFound,
        LIBUSB_ERROR_BUSY => Error::Busy,
        LIBUSB_ERROR_TIMEOUT => Error::Timeout,
        LIBUSB_ERROR_OVERFLOW => Error::Overflow,
        LIBUSB_ERROR_PIPE => Error::Pipe,
        LIBUSB_ERROR_INTERRUPTED => Error::Interrupted,
        LIBUSB_ERROR_NO_MEM => Error::NoMem,
        LIBUSB_ERROR_NOT_SUPPORTED => Error::NotSupported,
        LIBUSB_ERROR_OTHER | _ => Error::Other,
    }
}

#[doc(hidden)]
macro_rules! try_unsafe {
    ($x:expr) => {
        match unsafe { $x } {
            0 => (),
            err => return Err($crate::error::from_libusb(err)),
        }
    };
}
