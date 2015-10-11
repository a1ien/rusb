use std::mem;

use ::context::Context;
use ::device_handle::DeviceHandle;
use ::device::Device;
use ::configuration::Configuration;


/// A reference to a USB device.
pub struct DeviceRef<'a> {
    _context: &'a Context,
    device: *mut ::ffi::libusb_device
}

impl<'a> Drop for DeviceRef<'a> {
    /// Releases the device reference.
    fn drop(&mut self) {
        unsafe {
            ::ffi::libusb_unref_device(self.device);
        }
    }
}

impl<'a> DeviceRef<'a> {
    /// Reads information about the device.
    pub fn read_device(&mut self) -> ::Result<Device> {
        let mut descriptor: ::ffi::libusb_device_descriptor = unsafe { mem::uninitialized() };

        // since libusb 1.0.16, this function always succeeds
        try_unsafe!(::ffi::libusb_get_device_descriptor(self.device, &mut descriptor));

        let mut configurations = Vec::<Configuration>::with_capacity(descriptor.bNumConfigurations as usize);

        for i in (0..descriptor.bNumConfigurations) {
            let mut ptr: *const ::ffi::libusb_config_descriptor = unsafe { mem::uninitialized() };

            try_unsafe!(::ffi::libusb_get_config_descriptor(self.device, i, &mut ptr));

            configurations.push(::configuration::from_libusb(unsafe { mem::transmute(ptr) }));

            unsafe {
                ::ffi::libusb_free_config_descriptor(ptr);
            }
        }

        Ok(
            ::device::from_libusb(
                &descriptor,
                configurations,
                unsafe { ::ffi::libusb_get_bus_number(self.device) },
                unsafe { ::ffi::libusb_get_device_address(self.device) },
                unsafe { ::ffi::libusb_get_device_speed(self.device) }
            )
        )
    }

    /// Opens the device.
    pub fn open(&mut self) -> ::Result<DeviceHandle<'a>> {
        let mut handle: *mut ::ffi::libusb_device_handle = unsafe { mem::uninitialized() };

        try_unsafe!(::ffi::libusb_open(self.device, &mut handle));

        Ok(::device_handle::from_libusb(self._context, handle))
    }
}

#[doc(hidden)]
pub fn from_libusb<'a>(context: &'a Context, device: *mut ::ffi::libusb_device) -> DeviceRef<'a> {
    unsafe { ::ffi::libusb_ref_device(device) };

    DeviceRef {
        _context: context,
        device: device
    }
}
