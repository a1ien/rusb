use std::mem;

use libc::{c_int};

use ::error::UsbResult;
use ::device_list::DeviceList;


/// Library logging levels.
pub enum LogLevel {
  /// No messages are printed by `libusb` (default).
  None = ::ffi::LIBUSB_LOG_LEVEL_NONE as isize,

  /// Error messages printed to `stderr`.
  Error = ::ffi::LIBUSB_LOG_LEVEL_ERROR as isize,

  /// Warning and error messages are printed to `stderr`.
  Warning = ::ffi::LIBUSB_LOG_LEVEL_WARNING as isize,

  /// Informational messages are printed to `stdout`. Warnings and error messages are printed to
  /// `stderr`.
  Info = ::ffi::LIBUSB_LOG_LEVEL_INFO as isize,

  /// Debug and informational messages are printed to `stdout`. Warnings and error messages are
  /// printed to `stderr`.
  Debug = ::ffi::LIBUSB_LOG_LEVEL_DEBUG as isize
}


/// A `libusb` context.
pub struct Context {
  context: *mut ::ffi::libusb_context
}

impl Drop for Context {
  /// Closes the `libusb` context.
  fn drop(&mut self) {
    unsafe { ::ffi::libusb_exit(self.context) };
  }
}

impl Context {
  /// Opens a new `libusb` context.
  pub fn new() -> UsbResult<Self> {
    let mut context: *mut ::ffi::libusb_context = unsafe { mem::uninitialized() };

    match unsafe { ::ffi::libusb_init(&mut context) } {
      0 => Ok(Context { context: unsafe { mem::transmute(context) } }),
      e => Err(::error::from_libusb(e))
    }
  }

  /// Sets the log level of a `libusb` context.
  pub fn set_log_level(&mut self, level: LogLevel) {
    unsafe { ::ffi::libusb_set_debug(self.context, level as c_int) };
  }

  pub fn has_capability(&self) -> bool {
    match unsafe { ::ffi::libusb_has_capability(::ffi::LIBUSB_CAP_HAS_CAPABILITY) } {
      0 => false,
      _ => true
    }
  }

  /// Tests whether the running `libusb` library supports hotplug.
  pub fn has_hotplug(&self) -> bool {
    match unsafe { ::ffi::libusb_has_capability(::ffi::LIBUSB_CAP_HAS_HOTPLUG) } {
      0 => false,
      _ => true
    }
  }

  /// Tests whether the running `libusb` library has HID access.
  pub fn has_hid_access(&self) -> bool {
    match unsafe { ::ffi::libusb_has_capability(::ffi::LIBUSB_CAP_HAS_HID_ACCESS) } {
      0 => false,
      _ => true
    }
  }

  /// Tests whether the running `libusb` library supports detaching the kernel driver.
  pub fn supports_detach_kernel_driver(&self) -> bool {
    match unsafe { ::ffi::libusb_has_capability(::ffi::LIBUSB_CAP_SUPPORTS_DETACH_KERNEL_DRIVER) } {
      0 => false,
      _ => true
    }
  }

  /// Returns a list of the current USB devices. The context must outlive the device list.
  pub fn devices<'a>(&'a mut self) -> UsbResult<DeviceList<'a>> {
    let mut list: *const *mut ::ffi::libusb_device = unsafe { mem::uninitialized() };

    let n = unsafe { ::ffi::libusb_get_device_list(self.context, &mut list) };

    if n < 0 {
      Err(::error::from_libusb(n as c_int))
    }
    else {
      Ok(::device_list::from_libusb(self, list, n as usize))
    }
  }
}
