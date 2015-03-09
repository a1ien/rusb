use std::mem;

use libc::{c_int};

use ::error::UsbResult;
use ::context::Context;
use ::interface_handle::InterfaceHandle;


/// A handle to an open USB device.
pub struct DeviceHandle<'a> {
  _context: &'a Context,
  handle: *mut ::ffi::libusb_device_handle
}

#[unsafe_destructor]
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
}


// Not exported outside the crate.
pub fn from_libusb<'a>(context: &'a Context, handle: *mut ::ffi::libusb_device_handle) -> DeviceHandle<'a> {
  DeviceHandle {
    _context: context,
    handle: handle
  }
}
