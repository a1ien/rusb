use std::mem;

use libc::{c_int,c_uint,c_uchar};
use time::Duration;

use ::error::UsbResult;
use ::context::Context;
use ::interface_handle::InterfaceHandle;
use ::request::ControlRequest;

/// A handle to an open USB device.
pub struct DeviceHandle<'a> {
  _context: &'a Context,
  handle: *mut ::ffi::libusb_device_handle
}

impl<'a> Drop for DeviceHandle<'a> {
  /// Closes the device.
  fn drop(&mut self) {
    unsafe { ::ffi::libusb_close(self.handle) };
  }
}

impl<'a> DeviceHandle<'a> {
  /// Returns the active configuration number.
  pub fn active_configuration(&mut self) -> UsbResult<u8> {
    let mut config: c_int = unsafe { mem::uninitialized() };

    match unsafe { ::ffi::libusb_get_configuration(self.handle, &mut config) } {
      0 => Ok(config as u8),
      e => Err(::error::from_libusb(e))
    }
  }

  /// Sets the device's active configuration.
  pub fn set_active_configuration(&mut self, config: u8) -> UsbResult<()> {
    match unsafe { ::ffi::libusb_set_configuration(self.handle, config as c_int) } {
      0 => Ok(()),
      e => Err(::error::from_libusb(e))
    }
  }

  /// Puts the device in an unconfigured state.
  pub fn unconfigure(&mut self) -> UsbResult<()> {
    match unsafe { ::ffi::libusb_set_configuration(self.handle, -1) } {
      0 => Ok(()),
      e => Err(::error::from_libusb(e))
    }
  }

  /// Resets the device.
  pub fn reset(&mut self) -> UsbResult<()> {
    match unsafe { ::ffi::libusb_reset_device(self.handle) } {
      0 => Ok(()),
      e => Err(::error::from_libusb(e))
    }
  }

  /// Indicates whether the device has an attached kernel driver.
  ///
  /// This method is not supported on all platforms.
  pub fn kernel_driver_active(&mut self, iface: u8) -> UsbResult<bool> {
    match unsafe { ::ffi::libusb_kernel_driver_active(self.handle, iface as c_int) } {
      0 => Ok(false),
      1 => Ok(true),
      e => Err(::error::from_libusb(e))
    }
  }

  /// Detaches an attached kernel driver from the device.
  ///
  /// This method is not supported on all platforms.
  pub fn detach_kernel_driver(&mut self, iface: u8) -> UsbResult<()> {
    match unsafe { ::ffi::libusb_detach_kernel_driver(self.handle, iface as c_int) } {
      0 => Ok(()),
      e => Err(::error::from_libusb(e))
    }
  }

  /// Attaches a kernel driver to the device.
  ///
  /// This method is not supported on all platforms.
  pub fn attach_kernel_driver(&mut self, iface: u8) -> UsbResult<()> {
    match unsafe { ::ffi::libusb_attach_kernel_driver(self.handle, iface as c_int) } {
      0 => Ok(()),
      e => Err(::error::from_libusb(e))
    }
  }

  /// Claims one of the device's interfaces.
  ///
  /// An interface must be claimed before operating on it.
  pub fn claim_interface<'b>(&'b mut self, iface: u8) -> UsbResult<InterfaceHandle<'b>> {
    match unsafe { ::ffi::libusb_claim_interface(self.handle, iface as c_int) } {
      0 => Ok(::interface_handle::from_libusb(&self.handle, iface as c_int)),
      e => Err(::error::from_libusb(e))
    }
  }

  /// Performs a control transfer on the device.
  pub fn control_transfer(&mut self, request_type: ControlRequest, request: u8, value: u16, index: u16, data: &mut [u8], timeout: Duration) -> UsbResult<usize> {
    let buf = data.as_mut_ptr() as *mut c_uchar;
    let len = data.len() as u16;
    let timeout_ms = timeout.num_milliseconds() as c_uint;

    let res = unsafe { ::ffi::libusb_control_transfer(self.handle, request_type.to_u8(), request, value, index, buf, len, timeout_ms) };

    // LIBUSB_ERROR are negative integers
    if res < 0 {
      Err(::error::from_libusb(res))
    } else {
      Ok(res as usize)
    }
  }
}

// Not exported outside the crate.
pub fn from_libusb<'a>(context: &'a Context, handle: *mut ::ffi::libusb_device_handle) -> DeviceHandle<'a> {
  DeviceHandle {
    _context: context,
    handle: handle
  }
}
