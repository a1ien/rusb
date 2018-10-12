use std::marker::PhantomData;
use std::slice;

use libusb_sys::*;

use crate::context::Context;
use crate::device::{self, Device};

/// A list of detected USB devices.
pub struct DeviceList<'a> {
    context: PhantomData<&'a Context>,
    list: *const *mut libusb_device,
    len: usize,
}

impl<'a> Drop for DeviceList<'a> {
    /// Frees the device list.
    fn drop(&mut self) {
        unsafe {
            libusb_free_device_list(self.list, 1);
        }
    }
}

impl<'a> DeviceList<'a> {
    /// Returns the number of devices in the list.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns an iterator over the devices in the list.
    ///
    /// The iterator yields a sequence of `Device` objects.
    pub fn iter<'b>(&'b self) -> Devices<'a, 'b> {
        Devices {
            context: self.context,
            devices: unsafe { slice::from_raw_parts(self.list, self.len) },
            index: 0,
        }
    }
}

/// Iterator over detected USB devices.
pub struct Devices<'a, 'b> {
    context: PhantomData<&'a Context>,
    devices: &'b [*mut libusb_device],
    index: usize,
}

impl<'a, 'b> Iterator for Devices<'a, 'b> {
    type Item = Device<'a>;

    fn next(&mut self) -> Option<Device<'a>> {
        if self.index < self.devices.len() {
            let device = self.devices[self.index];

            self.index += 1;
            Some(unsafe { device::from_libusb(self.context, device) })
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.devices.len() - self.index;
        (remaining, Some(remaining))
    }
}

#[doc(hidden)]
pub unsafe fn from_libusb(
    _context: &Context,
    list: *const *mut libusb_device,
    len: usize,
) -> DeviceList {
    DeviceList {
        context: PhantomData,
        list,
        len,
    }
}
