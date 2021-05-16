use libc::c_int;

use std::{mem, slice};

use crate::{
    context::{GlobalContext, UsbContext},
    device::{self, Device},
    error,
};
use libusb1_sys::*;

/// A list of detected USB devices.
pub struct DeviceList<T: UsbContext> {
    context: T,
    list: *const *mut libusb_device,
    len: usize,
}

impl<T: UsbContext> Drop for DeviceList<T> {
    /// Frees the device list.
    fn drop(&mut self) {
        unsafe {
            libusb_free_device_list(self.list, 1);
        }
    }
}

impl DeviceList<GlobalContext> {
    pub fn new() -> crate::Result<DeviceList<GlobalContext>> {
        let mut list = mem::MaybeUninit::<*const *mut libusb_device>::uninit();

        let n =
            unsafe { libusb_get_device_list(GlobalContext::default().as_raw(), list.as_mut_ptr()) };

        if n < 0 {
            Err(error::from_libusb(n as c_int))
        } else {
            Ok(unsafe {
                DeviceList {
                    context: Default::default(),
                    list: list.assume_init(),
                    len: n as usize,
                }
            })
        }
    }
}

impl<T: UsbContext> DeviceList<T> {
    pub fn new_with_context(context: T) -> crate::Result<DeviceList<T>> {
        let mut list = mem::MaybeUninit::<*const *mut libusb_device>::uninit();

        let len = unsafe { libusb_get_device_list(context.as_raw(), list.as_mut_ptr()) };

        if len < 0 {
            Err(error::from_libusb(len as c_int))
        } else {
            Ok(unsafe {
                DeviceList {
                    context,
                    list: list.assume_init(),
                    len: len as usize,
                }
            })
        }
    }

    /// Returns the number of devices in the list.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true if the list is empty, else returns false.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns an iterator over the devices in the list.
    ///
    /// The iterator yields a sequence of `Device` objects.
    pub fn iter(&self) -> Devices<T> {
        Devices {
            context: self.context.clone(),
            devices: unsafe { slice::from_raw_parts(self.list, self.len) },
            index: 0,
        }
    }
}

/// Iterator over detected USB devices.
pub struct Devices<'a, T> {
    context: T,
    devices: &'a [*mut libusb_device],
    index: usize,
}

impl<'a, T: UsbContext> Iterator for Devices<'a, T> {
    type Item = Device<T>;

    fn next(&mut self) -> Option<Device<T>> {
        if self.index < self.devices.len() {
            let device = self.devices[self.index];

            self.index += 1;
            Some(unsafe {
                device::Device::from_libusb(
                    self.context.clone(),
                    std::ptr::NonNull::new_unchecked(device),
                )
            })
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.devices.len() - self.index;
        (remaining, Some(remaining))
    }
}
