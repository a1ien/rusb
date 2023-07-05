use std::{
    fmt::{self, Debug},
    mem,
    ptr::NonNull,
};

use libusb1_sys::*;

use crate::{
    config_descriptor::{self, ConfigDescriptor},
    device_descriptor::{self, DeviceDescriptor},
    device_handle::DeviceHandle,
    error,
    fields::{self, Speed},
    Error, Result, Context,
};

/// A reference to a USB device.
#[derive(Eq, PartialEq)]
pub struct Device {
    context: Context,
    device: NonNull<libusb_device>,
}

impl Drop for Device {
    /// Releases the device reference.
    fn drop(&mut self) {
        unsafe {
            libusb_unref_device(self.device.as_ptr());
        }
    }
}

impl Clone for Device {
    fn clone(&self) -> Self {
        unsafe { Self::from_libusb(self.context.clone(), self.device) }
    }
}

unsafe impl Send for Device {}
unsafe impl Sync for Device {}

impl Debug for Device {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let descriptor = match self.device_descriptor() {
            Ok(descriptor) => descriptor,
            Err(e) => {
                return write!(f, "Can't read device descriptor {:?}", e);
            }
        };
        write!(
            f,
            "Bus {:03} Device {:03}: ID {:04x}:{:04x}",
            self.bus_number(),
            self.address(),
            descriptor.vendor_id(),
            descriptor.product_id(),
        )
    }
}

impl Device {
    /// Get the raw libusb_device pointer, for advanced use in unsafe code
    pub fn as_raw(&self) -> *mut libusb_device {
        self.device.as_ptr()
    }

    /// Get the context associated with this device
    pub fn context(&self) -> &Context {
        &self.context
    }

    /// # Safety
    ///
    /// Converts an existing `libusb_device` pointer into a `Device`.
    /// `device` must be a pointer to a valid `libusb_device`. Rusb increments refcount.
    pub unsafe fn from_libusb(context: Context, device: NonNull<libusb_device>) -> Self {
        libusb_ref_device(device.as_ptr());
        Self { context, device }
    }

    /// Reads the device descriptor.
    pub fn device_descriptor(&self) -> Result<DeviceDescriptor> {
        let mut descriptor = mem::MaybeUninit::<libusb_device_descriptor>::uninit();

        // since libusb 1.0.16, this function always succeeds
        try_unsafe!(libusb_get_device_descriptor(
            self.device.as_ptr(),
            descriptor.as_mut_ptr()
        ));

        Ok(device_descriptor::from_libusb(unsafe {
            descriptor.assume_init()
        }))
    }

    /// Reads a configuration descriptor.
    pub fn config_descriptor(&self, config_index: u8) -> Result<ConfigDescriptor> {
        let mut config = mem::MaybeUninit::<*const libusb_config_descriptor>::uninit();

        try_unsafe!(libusb_get_config_descriptor(
            self.device.as_ptr(),
            config_index,
            config.as_mut_ptr()
        ));

        Ok(unsafe { config_descriptor::from_libusb(config.assume_init()) })
    }

    /// Reads the configuration descriptor for the current configuration.
    pub fn active_config_descriptor(&self) -> Result<ConfigDescriptor> {
        let mut config = mem::MaybeUninit::<*const libusb_config_descriptor>::uninit();

        try_unsafe!(libusb_get_active_config_descriptor(
            self.device.as_ptr(),
            config.as_mut_ptr()
        ));

        Ok(unsafe { config_descriptor::from_libusb(config.assume_init()) })
    }

    /// Returns the number of the bus that the device is connected to.
    pub fn bus_number(&self) -> u8 {
        unsafe { libusb_get_bus_number(self.device.as_ptr()) }
    }

    /// Returns the device's address on the bus that it's connected to.
    pub fn address(&self) -> u8 {
        unsafe { libusb_get_device_address(self.device.as_ptr()) }
    }

    /// Returns the device's connection speed.
    pub fn speed(&self) -> Speed {
        fields::speed_from_libusb(unsafe { libusb_get_device_speed(self.device.as_ptr()) })
    }

    /// Opens the device.
    pub fn open(&self) -> Result<DeviceHandle> {
        let mut handle = mem::MaybeUninit::<*mut libusb_device_handle>::uninit();

        try_unsafe!(libusb_open(self.device.as_ptr(), handle.as_mut_ptr()));

        Ok(unsafe {
            let ptr = NonNull::new(handle.assume_init()).ok_or(Error::NoDevice)?;
            DeviceHandle::from_libusb(self.context.clone(), ptr)
        })
    }

    /// Returns the device's port number
    pub fn port_number(&self) -> u8 {
        unsafe { libusb_get_port_number(self.device.as_ptr()) }
    }

    /// Returns the device's parent
    pub fn get_parent(&self) -> Option<Self> {
        let device = unsafe { libusb_get_parent(self.device.as_ptr()) };
        NonNull::new(device)
            .map(|device| unsafe { Device::from_libusb(self.context.clone(), device) })
    }

    ///  Get the list of all port numbers from root for the specified device
    pub fn port_numbers(&self) -> Result<Vec<u8>> {
        // As per the USB 3.0 specs, the current maximum limit for the depth is 7.
        let mut ports = [0; 7];

        let result = unsafe {
            libusb_get_port_numbers(self.device.as_ptr(), ports.as_mut_ptr(), ports.len() as i32)
        };

        let ports_number = if result < 0 {
            return Err(error::from_libusb(result));
        } else {
            result
        };
        Ok(ports[0..ports_number as usize].to_vec())
    }
}
