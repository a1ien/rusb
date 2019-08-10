use std::marker::PhantomData;
use std::mem;

use libusb1_sys::*;

use crate::config_descriptor::{self, ConfigDescriptor};
use crate::context::Context;
use crate::device_descriptor::{self, DeviceDescriptor};
use crate::device_handle::{self, DeviceHandle};
use crate::fields::{self, Speed};

/// A reference to a USB device.
pub struct Device<'a> {
    context: PhantomData<&'a Context>,
    device: *mut libusb_device,
}

impl<'a> Drop for Device<'a> {
    /// Releases the device reference.
    fn drop(&mut self) {
        unsafe {
            libusb_unref_device(self.device);
        }
    }
}

unsafe impl<'a> Send for Device<'a> {}
unsafe impl<'a> Sync for Device<'a> {}

impl<'a> Device<'a> {
    /// Get the raw libusb_device pointer, for advanced use in unsafe code
    pub fn as_raw(&self) -> *mut libusb_device {
        self.device
    }

    /// Reads the device descriptor.
    pub fn device_descriptor(&self) -> crate::Result<DeviceDescriptor> {
        let mut descriptor = mem::MaybeUninit::<libusb_device_descriptor>::uninit();

        // since libusb 1.0.16, this function always succeeds
        try_unsafe!(libusb_get_device_descriptor(
            self.device,
            descriptor.as_mut_ptr()
        ));

        Ok(device_descriptor::from_libusb(unsafe {
            descriptor.assume_init()
        }))
    }

    /// Reads a configuration descriptor.
    pub fn config_descriptor(&self, config_index: u8) -> crate::Result<ConfigDescriptor> {
        let mut config = mem::MaybeUninit::<*const libusb_config_descriptor>::uninit();

        try_unsafe!(libusb_get_config_descriptor(
            self.device,
            config_index,
            config.as_mut_ptr()
        ));

        Ok(unsafe { config_descriptor::from_libusb(config.assume_init()) })
    }

    /// Reads the configuration descriptor for the current configuration.
    pub fn active_config_descriptor(&self) -> crate::Result<ConfigDescriptor> {
        let mut config = mem::MaybeUninit::<*const libusb_config_descriptor>::uninit();

        try_unsafe!(libusb_get_active_config_descriptor(
            self.device,
            config.as_mut_ptr()
        ));

        Ok(unsafe { config_descriptor::from_libusb(config.assume_init()) })
    }

    /// Returns the number of the bus that the device is connected to.
    pub fn bus_number(&self) -> u8 {
        unsafe { libusb_get_bus_number(self.device) }
    }

    /// Returns the device's address on the bus that it's connected to.
    pub fn address(&self) -> u8 {
        unsafe { libusb_get_device_address(self.device) }
    }

    /// Returns the device's connection speed.
    pub fn speed(&self) -> Speed {
        fields::speed_from_libusb(unsafe { libusb_get_device_speed(self.device) })
    }

    /// Opens the device.
    pub fn open(&self) -> crate::Result<DeviceHandle<'a>> {
        let mut handle = mem::MaybeUninit::<*mut libusb_device_handle>::uninit();

        try_unsafe!(libusb_open(self.device, handle.as_mut_ptr()));

        Ok(unsafe { device_handle::from_libusb(self.context, handle.assume_init()) })
    }

    /// Returns the device's port number
    pub fn port_number(&self) -> u8 {
        unsafe { libusb_get_port_number(self.device) }
    }
}

#[doc(hidden)]
pub(crate) unsafe fn from_libusb(
    context: PhantomData<&Context>,
    device: *mut libusb_device,
) -> Device {
    libusb_ref_device(device);

    Device { context, device }
}
