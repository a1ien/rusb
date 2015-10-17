use std::marker::PhantomData;
use std::mem;

use ::context::Context;
use ::device_handle::DeviceHandle;
use ::device_descriptor::DeviceDescriptor;
use ::config_descriptor::ConfigDescriptor;
use ::fields::Speed;


/// A reference to a USB device.
pub struct Device<'a> {
    context: PhantomData<&'a Context>,
    device: *mut ::libusb::libusb_device
}

impl<'a> Drop for Device<'a> {
    /// Releases the device reference.
    fn drop(&mut self) {
        unsafe {
            ::libusb::libusb_unref_device(self.device);
        }
    }
}

impl<'a> Device<'a> {
    /// Reads the device descriptor.
    pub fn device_descriptor(&mut self) -> ::Result<DeviceDescriptor> {
        let mut descriptor: ::libusb::libusb_device_descriptor = unsafe { mem::uninitialized() };

        // since libusb 1.0.16, this function always succeeds
        try_unsafe!(::libusb::libusb_get_device_descriptor(self.device, &mut descriptor));

        Ok(::device_descriptor::from_libusb(descriptor))
    }

    /// Reads a configuration descriptor.
    pub fn config_descriptor(&mut self, config_index: u8) -> ::Result<ConfigDescriptor> {
        let mut config: *const ::libusb::libusb_config_descriptor = unsafe { mem::uninitialized() };

        try_unsafe!(::libusb::libusb_get_config_descriptor(self.device, config_index, &mut config));

        Ok(::config_descriptor::from_libusb(config))
    }

    /// Returns the number of the bus that the device is connected to.
    pub fn bus_number(&mut self) -> u8 {
        unsafe {
            ::libusb::libusb_get_bus_number(self.device)
        }
    }

    /// Returns the device's address on the bus that it's connected to.
    pub fn address(&mut self) -> u8 {
        unsafe {
            ::libusb::libusb_get_device_address(self.device)
        }
    }

    /// Returns the device's connection speed.
    pub fn speed(&mut self) -> Speed {
        ::fields::speed_from_libusb(unsafe {
            ::libusb::libusb_get_device_speed(self.device)
        })
    }

    /// Opens the device.
    pub fn open(&mut self) -> ::Result<DeviceHandle<'a>> {
        let mut handle: *mut ::libusb::libusb_device_handle = unsafe { mem::uninitialized() };

        try_unsafe!(::libusb::libusb_open(self.device, &mut handle));

        Ok(::device_handle::from_libusb(self.context, handle))
    }
}

#[doc(hidden)]
pub fn from_libusb<'a>(context: PhantomData<&'a Context>, device: *mut ::libusb::libusb_device) -> Device<'a> {
    unsafe { ::libusb::libusb_ref_device(device) };

    Device {
        context: context,
        device: device,
    }
}
