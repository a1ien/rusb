use std::mem;
use std::slice;

use libc::{c_int,c_uint,c_uchar};
use time::Duration;

use ::context::Context;
use ::interface_handle::InterfaceHandle;
use ::device::Device;
use ::configuration::Configuration;
use ::interface::InterfaceSetting;
use ::request::{ControlRequest,Direction,RequestType,Recipient};
use ::language::Language;

/// A handle to an open USB device.
pub struct DeviceHandle<'a> {
    _context: &'a Context,
    handle: *mut ::ffi::libusb_device_handle
}

impl<'a> Drop for DeviceHandle<'a> {
    /// Closes the device.
    fn drop(&mut self) {
        unsafe {
            ::ffi::libusb_close(self.handle);
        }
    }
}

impl<'a> DeviceHandle<'a> {
    /// Returns the active configuration number.
    pub fn active_configuration(&mut self) -> ::Result<u8> {
        let mut config = unsafe { mem::uninitialized() };

        try_unsafe!(::ffi::libusb_get_configuration(self.handle, &mut config));
        Ok(config as u8)
    }

    /// Sets the device's active configuration.
    pub fn set_active_configuration(&mut self, config: u8) -> ::Result<()> {
        try_unsafe!(::ffi::libusb_set_configuration(self.handle, config as c_int));
        Ok(())
    }

    /// Puts the device in an unconfigured state.
    pub fn unconfigure(&mut self) -> ::Result<()> {
        try_unsafe!(::ffi::libusb_set_configuration(self.handle, -1));
        Ok(())
    }

    /// Resets the device.
    pub fn reset(&mut self) -> ::Result<()> {
        try_unsafe!(::ffi::libusb_reset_device(self.handle));
        Ok(())
    }

    /// Indicates whether the device has an attached kernel driver.
    ///
    /// This method is not supported on all platforms.
    pub fn kernel_driver_active(&mut self, iface: u8) -> ::Result<bool> {
        match unsafe { ::ffi::libusb_kernel_driver_active(self.handle, iface as c_int) } {
            0 => Ok(false),
            1 => Ok(true),
            err => Err(::error::from_libusb(err))
        }
    }

    /// Detaches an attached kernel driver from the device.
    ///
    /// This method is not supported on all platforms.
    pub fn detach_kernel_driver(&mut self, iface: u8) -> ::Result<()> {
        try_unsafe!(::ffi::libusb_detach_kernel_driver(self.handle, iface as c_int));
        Ok(())
    }

    /// Attaches a kernel driver to the device.
    ///
    /// This method is not supported on all platforms.
    pub fn attach_kernel_driver(&mut self, iface: u8) -> ::Result<()> {
        try_unsafe!(::ffi::libusb_attach_kernel_driver(self.handle, iface as c_int));
        Ok(())
    }

    /// Claims one of the device's interfaces.
    ///
    /// An interface must be claimed before operating on it.
    pub fn claim_interface<'b>(&'b mut self, iface: u8) -> ::Result<InterfaceHandle<'b>> {
        try_unsafe!(::ffi::libusb_claim_interface(self.handle, iface as c_int));
        Ok(::interface_handle::from_libusb(&self.handle, iface as c_int))
    }

    /// Performs a control transfer on the device.
    pub fn control_transfer(&mut self, request_type: ControlRequest, request: u8, value: u16, index: u16, data: &mut [u8], timeout: Duration) -> ::Result<usize> {
        let buf = data.as_mut_ptr() as *mut c_uchar;
        let len = data.len() as u16;
        let timeout_ms = timeout.num_milliseconds() as c_uint;

        let res = unsafe {
            ::ffi::libusb_control_transfer(self.handle, request_type.to_u8(), request, value, index, buf, len, timeout_ms)
        };

        if res < 0 {
            Err(::error::from_libusb(res))
        } else {
            Ok(res as usize)
        }
    }

    /// Reads the languages supported by the device's string descriptors.
    ///
    /// This function returns a list of languages that can be used to read the device's string
    /// descriptors.
    pub fn read_languages(&mut self, timeout: Duration) -> ::Result<Vec<Language>> {
        let mut buf = Vec::<u8>::with_capacity(256);

        let request = ControlRequest::new(Direction::In, RequestType::Standard, Recipient::Device);
        let mut buf_slice = unsafe {
            slice::from_raw_parts_mut((&mut buf[..]).as_mut_ptr(), buf.capacity())
        };

        let len = try!(self.control_transfer(request, ::ffi::LIBUSB_REQUEST_GET_DESCRIPTOR, (::ffi::LIBUSB_DT_STRING as u16) << 8, 0, buf_slice, timeout));

        unsafe {
            buf.set_len(len);
        }

        Ok(buf.chunks(2).skip(1).map(|chunk| {
            let lang_id = chunk[0] as u16 | (chunk[1] as u16) << 8;
            ::language::from_lang_id(lang_id)
        }).collect())
    }

    /// Reads a string descriptor from the device.
    ///
    /// `language` should be one of the languages returned from [`read_languages`](#method.read_languages).
    pub fn read_string_descriptor(&mut self, language: Language, index: u8, timeout: Duration) -> ::Result<String> {
        let mut buf = Vec::<u8>::with_capacity(256);

        let request = ControlRequest::new(Direction::In, RequestType::Standard, Recipient::Device);
        let mut buf_slice = unsafe {
            slice::from_raw_parts_mut((&mut buf[..]).as_mut_ptr(), buf.capacity())
        };

        let len = try!(self.control_transfer(request, ::ffi::LIBUSB_REQUEST_GET_DESCRIPTOR, (::ffi::LIBUSB_DT_STRING as u16) << 8 | index as u16, language.lang_id(), buf_slice, timeout));

        unsafe {
            buf.set_len(len);
        }

        let utf16: Vec<u16> = buf.chunks(2).skip(1).map(|chunk| {
            chunk[0] as u16 | (chunk[1] as u16) << 8
        }).collect();

        String::from_utf16(&utf16[..]).map_err(|_| ::error::Error::Other)
    }

    /// Reads the device's manufacturer string descriptor.
    pub fn read_manufacturer_string(&mut self, language: Language, device: &Device, timeout: Duration) -> ::Result<String> {
        match device.manufacturer_string_index() {
            None => Err(::Error::InvalidParam),
            Some(n) => self.read_string_descriptor(language, n, timeout)
        }
    }

    /// Reads the device's product string descriptor.
    pub fn read_product_string(&mut self, language: Language, device: &Device, timeout: Duration) -> ::Result<String> {
        match device.product_string_index() {
            None => Err(::Error::InvalidParam),
            Some(n) => self.read_string_descriptor(language, n, timeout)
        }
    }

    /// Reads the device's serial number string descriptor.
    pub fn read_serial_number_string(&mut self, language: Language, device: &Device, timeout: Duration) -> ::Result<String> {
        match device.serial_number_string_index() {
            None => Err(::Error::InvalidParam),
            Some(n) => self.read_string_descriptor(language, n, timeout)
        }
    }

    /// Reads the string descriptor for a configuration's description.
    pub fn read_configuration_string(&mut self, language: Language, configuration: &Configuration, timeout: Duration) -> ::Result<String> {
        match configuration.description_string_index() {
            None => Err(::Error::InvalidParam),
            Some(n) => self.read_string_descriptor(language, n, timeout)
        }
    }

    /// Reads the string descriptor for a interface's description.
    pub fn read_interface_string(&mut self, language: Language, interface: &InterfaceSetting, timeout: Duration) -> ::Result<String> {
        match interface.description_string_index() {
            None => Err(::Error::InvalidParam),
            Some(n) => self.read_string_descriptor(language, n, timeout)
        }
    }
}

#[doc(hidden)]
pub fn from_libusb<'a>(context: &'a Context, handle: *mut ::ffi::libusb_device_handle) -> DeviceHandle<'a> {
    DeviceHandle {
        _context: context,
        handle: handle
    }
}
