use std::fmt;

use std::error::Error;
use std::fmt::{Display,Formatter};

use libc::{c_int};


/// A result of a function that may return a `UsbError`.
pub type UsbResult<T> = Result<T, UsbError>;


/// Errors returned by the `libusb` library.
#[derive(Debug)]
pub enum UsbError {
  /// Success (no error).
  Success,

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

  /// Other error.
  Other
}

impl UsbError {
  /// Returns a description of an error suitable for display to an end user.
  pub fn strerror(&self) -> &'static str {
    match *self {
      UsbError::Success      => "Success",
      UsbError::Io           => "Input/Output Error",
      UsbError::InvalidParam => "Invalid parameter",
      UsbError::Access       => "Access denied (insufficient permissions)",
      UsbError::NoDevice     => "No such device (it may have been disconnected)",
      UsbError::NotFound     => "Entity not found",
      UsbError::Busy         => "Resource busy",
      UsbError::Timeout      => "Operation timed out",
      UsbError::Overflow     => "Overflow",
      UsbError::Pipe         => "Pipe error",
      UsbError::Interrupted  => "System call interrupted (perhaps due to signal)",
      UsbError::NoMem        => "Insufficient memory",
      UsbError::NotSupported => "Operation not supported or unimplemented on this platform",
      UsbError::Other        => "Other error"
    }
  }
}

impl Display for UsbError {
  fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
    fmt.write_str(self.strerror())
  }
}

impl Error for UsbError {
  fn description(&self) -> &'static str {
    self.strerror()
  }
}


pub fn from_libusb(err: c_int) -> UsbError {
  match err {
    ::ffi::LIBUSB_SUCCESS             => UsbError::Success,
    ::ffi::LIBUSB_ERROR_IO            => UsbError::Io,
    ::ffi::LIBUSB_ERROR_INVALID_PARAM => UsbError::InvalidParam,
    ::ffi::LIBUSB_ERROR_ACCESS        => UsbError::Access,
    ::ffi::LIBUSB_ERROR_NO_DEVICE     => UsbError::NoDevice,
    ::ffi::LIBUSB_ERROR_NOT_FOUND     => UsbError::NotFound,
    ::ffi::LIBUSB_ERROR_BUSY          => UsbError::Busy,
    ::ffi::LIBUSB_ERROR_TIMEOUT       => UsbError::Timeout,
    ::ffi::LIBUSB_ERROR_OVERFLOW      => UsbError::Overflow,
    ::ffi::LIBUSB_ERROR_PIPE          => UsbError::Pipe,
    ::ffi::LIBUSB_ERROR_INTERRUPTED   => UsbError::Interrupted,
    ::ffi::LIBUSB_ERROR_NO_MEM        => UsbError::NoMem,
    ::ffi::LIBUSB_ERROR_NOT_SUPPORTED => UsbError::NotSupported,
    ::ffi::LIBUSB_ERROR_OTHER | _     => UsbError::Other
  }
}
