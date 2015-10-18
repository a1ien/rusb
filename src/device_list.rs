use std::marker::PhantomData;
use std::slice;

use ::context::Context;
use ::device::Device;


/// A list of detected USB devices.
pub struct DeviceList<'a> {
    context: PhantomData<&'a Context>,
    list: *const *mut ::libusb::libusb_device,
    len: usize,
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
    devices: &'b [*mut ::libusb::libusb_device],
    index: usize,
}

impl<'a, 'b> Iterator for Devices<'a, 'b> {
    type Item = Device<'a>;

    fn next(&mut self) -> Option<Device<'a>> {
        if self.index < self.devices.len() {
            let device = self.devices[self.index];

            self.index += 1;
            Some(::device::from_libusb(self.context, device))
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
pub fn from_libusb<'a>(_context: &'a Context, list: *const *mut ::libusb::libusb_device, len: usize,) -> DeviceList<'a> {
    DeviceList {
        context: PhantomData,
        list: list,
        len: len,
    }
}
