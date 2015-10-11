use std::mem;

use libc::{c_int,c_uint,c_uchar};
use time::Duration;

/// A handle to a claimed USB device interface.
pub struct InterfaceHandle<'a> {
    handle: &'a *mut ::libusb::libusb_device_handle,
    iface: c_int
}

impl<'a> Drop for InterfaceHandle<'a> {
    /// Releases the interface.
    fn drop(&mut self) {
        unsafe {
            ::libusb::libusb_release_interface(*self.handle, self.iface);
        }
    }
}

impl<'a> InterfaceHandle<'a> {
    /// Sets the interfaces active setting.
    pub fn set_alternate_setting(&mut self, setting: u8) -> ::Result<()> {
        try_unsafe!(::libusb::libusb_set_interface_alt_setting(*self.handle, self.iface, setting as c_int));
        Ok(())
    }

    /// Performs an interrupt transfer on one of the interface's endpoints.
    pub fn interrupt_transfer(&mut self, endpoint: u8, data: &mut [u8], timeout: Duration) -> ::Result<usize> {
        let mut transferred: c_int = unsafe { mem::uninitialized() };

        let buf = data.as_mut_ptr() as *mut c_uchar;
        let len = data.len() as c_int;
        let timeout_ms = timeout.num_milliseconds() as c_uint;

        try_unsafe!(::libusb::libusb_interrupt_transfer(*self.handle, endpoint, buf, len, &mut transferred, timeout_ms));
        Ok(transferred as usize)
    }

    /// Performs a bulk transfer on one of the devices endpoints.
    pub fn bulk_transfer(&mut self, endpoint: u8, data: &mut [u8], timeout: Duration) -> ::Result<usize> {
        let mut transferred: c_int = unsafe { mem::uninitialized() };

        let buf = data.as_mut_ptr() as *mut c_uchar;
        let len = data.len() as c_int;
        let timeout_ms = timeout.num_milliseconds() as c_uint;

        try_unsafe!(::libusb::libusb_bulk_transfer(*self.handle, endpoint, buf, len, &mut transferred, timeout_ms));
        Ok(transferred as usize)
    }
}

#[doc(hidden)]
pub fn from_libusb<'a>(handle: &'a *mut ::libusb::libusb_device_handle, iface: c_int) -> InterfaceHandle<'a> {
    InterfaceHandle {
        handle: handle,
        iface: iface
    }
}
