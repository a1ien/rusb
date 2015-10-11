use std::slice;

use ::context::Context;
use ::device_ref::DeviceRef;


/// A list of detected USB devices.
pub struct DeviceList<'a> {
    _context: &'a Context,
    list: *const *mut ::libusb::libusb_device,
    len: usize
}

impl<'a> Drop for DeviceList<'a> {
    /// Frees the device list.
    fn drop(&mut self) {
        unsafe {
            ::libusb::libusb_free_device_list(self.list, 1);
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
    /// The iterator yields a sequence of `DeviceRef` objects.
    pub fn iter<'b>(&'b self) -> Iter<'a, 'b> {
        Iter {
            _context: self._context,
            devices: unsafe { slice::from_raw_parts(self.list, self.len) },
            index: 0
        }
    }
}


/// Iterates over detected USB devices.
pub struct Iter<'a, 'b> {
    _context: &'a Context,
    devices: &'b [*mut ::libusb::libusb_device],
    index: usize
}

impl<'a, 'b> Iterator for Iter<'a, 'b> {
    type Item = DeviceRef<'a>;

    fn next(&mut self) -> Option<DeviceRef<'a>> {
        if self.index < self.devices.len() {
            let device = self.devices[self.index];

            self.index += 1;
            Some(::device_ref::from_libusb(self._context, device))
        }
        else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.devices.len() - self.index;
        (remaining, Some(remaining))
    }
}


#[doc(hidden)]
pub fn from_libusb<'a>(context: &'a Context, list: *const *mut ::libusb::libusb_device, len: usize,) -> DeviceList<'a> {
    DeviceList {
        _context: context,
        list: list,
        len: len
    }
}
