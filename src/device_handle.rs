use std::marker::PhantomData;
use std::mem;
use std::slice;
use std::time::Duration;

use bit_set::BitSet;
use libc::{c_int, c_uint, c_uchar};
use libusb::*;

use context::Context;
use error::{self, Error};
use device_descriptor::DeviceDescriptor;
use config_descriptor::ConfigDescriptor;
use interface_descriptor::InterfaceDescriptor;
use fields::{Direction, RequestType, Recipient, request_type};
use language::Language;

/// A handle to an open USB device.
pub struct DeviceHandle<'a> {
    _context: PhantomData<&'a Context>,
    handle: *mut libusb_device_handle,
    interfaces: BitSet,
}

impl<'a> Drop for DeviceHandle<'a> {
    /// Closes the device.
    fn drop(&mut self) {
        unsafe {
            for iface in self.interfaces.iter() {
                libusb_release_interface(self.handle, iface as c_int);
            }

            libusb_close(self.handle);
        }
    }
}

unsafe impl<'a> Send for DeviceHandle<'a> {}
unsafe impl<'a> Sync for DeviceHandle<'a> {}

impl<'a> DeviceHandle<'a> {
    /// Returns the active configuration number.
    pub fn active_configuration(&self) -> ::Result<u8> {
        let mut config = unsafe { mem::uninitialized() };

        try_unsafe!(libusb_get_configuration(self.handle, &mut config));
        Ok(config as u8)
    }

    /// Sets the device's active configuration.
    pub fn set_active_configuration(&mut self, config: u8) -> ::Result<()> {
        try_unsafe!(libusb_set_configuration(self.handle, config as c_int));
        Ok(())
    }

    /// Puts the device in an unconfigured state.
    pub fn unconfigure(&mut self) -> ::Result<()> {
        try_unsafe!(libusb_set_configuration(self.handle, -1));
        Ok(())
    }

    /// Resets the device.
    pub fn reset(&mut self) -> ::Result<()> {
        try_unsafe!(libusb_reset_device(self.handle));
        Ok(())
    }

    /// Indicates whether the device has an attached kernel driver.
    ///
    /// This method is not supported on all platforms.
    pub fn kernel_driver_active(&self, iface: u8) -> ::Result<bool> {
        match unsafe { libusb_kernel_driver_active(self.handle, iface as c_int) } {
            0 => Ok(false),
            1 => Ok(true),
            err => Err(error::from_libusb(err)),
        }
    }

    /// Detaches an attached kernel driver from the device.
    ///
    /// This method is not supported on all platforms.
    pub fn detach_kernel_driver(&mut self, iface: u8) -> ::Result<()> {
        try_unsafe!(libusb_detach_kernel_driver(self.handle, iface as c_int));
        Ok(())
    }

    /// Attaches a kernel driver to the device.
    ///
    /// This method is not supported on all platforms.
    pub fn attach_kernel_driver(&mut self, iface: u8) -> ::Result<()> {
        try_unsafe!(libusb_attach_kernel_driver(self.handle, iface as c_int));
        Ok(())
    }

    /// Claims one of the device's interfaces.
    ///
    /// An interface must be claimed before operating on it. All claimed interfaces are released
    /// when the device handle goes out of scope.
    pub fn claim_interface(&mut self, iface: u8) -> ::Result<()> {
        try_unsafe!(libusb_claim_interface(self.handle, iface as c_int));
        self.interfaces.insert(iface as usize);
        Ok(())
    }

    /// Releases a claimed interface.
    pub fn release_interface(&mut self, iface: u8) -> ::Result<()> {
        try_unsafe!(libusb_release_interface(self.handle, iface as c_int));
        self.interfaces.remove(iface as usize);
        Ok(())
    }

    /// Sets an interface's active setting.
    pub fn set_alternate_setting(&mut self, iface: u8, setting: u8) -> ::Result<()> {
        try_unsafe!(libusb_set_interface_alt_setting(self.handle, iface as c_int, setting as c_int));
        Ok(())
    }

    /// Reads from an interrupt endpoint.
    ///
    /// This function attempts to read from the interrupt endpoint with the address given by the
    /// `endpoint` parameter and fills `buf` with any data received from the endpoint. The function
    /// blocks up to the amount of time specified by `timeout`.
    ///
    /// If the return value is `Ok(n)`, then `buf` is populated with `n` bytes of data received
    /// from the endpoint.
    ///
    /// ## Errors
    ///
    /// If this function encounters any form of error while fulfilling the transfer request, an
    /// error variant will be returned. If an error variant is returned, no bytes were read.
    ///
    /// The errors returned by this function include:
    ///
    /// * `InvalidParam` if the endpoint is not an input endpoint.
    /// * `Timeout` if the transfer timed out.
    /// * `Pipe` if the endpoint halted.
    /// * `Overflow` if the device offered more data.
    /// * `NoDevice` if the device has been disconnected.
    /// * `Io` if the transfer encountered an I/O error.
    pub fn read_interrupt(&self, endpoint: u8, buf: &mut [u8], timeout: Duration) -> ::Result<usize> {
        if endpoint & LIBUSB_ENDPOINT_DIR_MASK != LIBUSB_ENDPOINT_IN {
            return Err(Error::InvalidParam);
        }

        let mut transferred: c_int = unsafe { mem::uninitialized() };

        let ptr = buf.as_mut_ptr() as *mut c_uchar;
        let len = buf.len() as c_int;
        let timeout_ms = (timeout.as_secs() * 1000 + timeout.subsec_nanos() as u64 / 1_000_000) as c_uint;

        match unsafe { libusb_interrupt_transfer(self.handle, endpoint, ptr, len, &mut transferred, timeout_ms) } {
            0 => {
                Ok(transferred as usize)
            },
            err => {
                if err == LIBUSB_ERROR_INTERRUPTED && transferred > 0 {
                    Ok(transferred as usize)
                }
                else {
                    Err(error::from_libusb(err))
                }
            },
        }
    }

    /// Writes to an interrupt endpoint.
    ///
    /// This function attempts to write the contents of `buf` to the interrupt endpoint with the
    /// address given by the `endpoint` parameter. The function blocks up to the amount of time
    /// specified by `timeout`.
    ///
    /// If the return value is `Ok(n)`, then `n` bytes of `buf` were written to the endpoint.
    ///
    /// ## Errors
    ///
    /// If this function encounters any form of error while fulfilling the transfer request, an
    /// error variant will be returned. If an error variant is returned, no bytes were written.
    ///
    /// The errors returned by this function include:
    ///
    /// * `InvalidParam` if the endpoint is not an output endpoint.
    /// * `Timeout` if the transfer timed out.
    /// * `Pipe` if the endpoint halted.
    /// * `NoDevice` if the device has been disconnected.
    /// * `Io` if the transfer encountered an I/O error.
    pub fn write_interrupt(&self, endpoint: u8, buf: &[u8], timeout: Duration) -> ::Result<usize> {
        if endpoint & LIBUSB_ENDPOINT_DIR_MASK != LIBUSB_ENDPOINT_OUT {
            return Err(Error::InvalidParam);
        }

        let mut transferred: c_int = unsafe { mem::uninitialized() };

        let ptr = buf.as_ptr() as *mut c_uchar;
        let len = buf.len() as c_int;
        let timeout_ms = (timeout.as_secs() * 1000 + timeout.subsec_nanos() as u64 / 1_000_000) as c_uint;

        match unsafe { libusb_interrupt_transfer(self.handle, endpoint, ptr, len, &mut transferred, timeout_ms) } {
            0 => {
                Ok(transferred as usize)
            },
            err => {
                if err == LIBUSB_ERROR_INTERRUPTED && transferred > 0 {
                    Ok(transferred as usize)
                }
                else {
                    Err(error::from_libusb(err))
                }
            },
        }
    }

    /// Reads from a bulk endpoint.
    ///
    /// This function attempts to read from the bulk endpoint with the address given by the
    /// `endpoint` parameter and fills `buf` with any data received from the endpoint. The function
    /// blocks up to the amount of time specified by `timeout`.
    ///
    /// If the return value is `Ok(n)`, then `buf` is populated with `n` bytes of data received
    /// from the endpoint.
    ///
    /// ## Errors
    ///
    /// If this function encounters any form of error while fulfilling the transfer request, an
    /// error variant will be returned. If an error variant is returned, no bytes were read.
    ///
    /// The errors returned by this function include:
    ///
    /// * `InvalidParam` if the endpoint is not an input endpoint.
    /// * `Timeout` if the transfer timed out.
    /// * `Pipe` if the endpoint halted.
    /// * `Overflow` if the device offered more data.
    /// * `NoDevice` if the device has been disconnected.
    /// * `Io` if the transfer encountered an I/O error.
    pub fn read_bulk(&self, endpoint: u8, buf: &mut [u8], timeout: Duration) -> ::Result<usize> {
        if endpoint & LIBUSB_ENDPOINT_DIR_MASK != LIBUSB_ENDPOINT_IN {
            return Err(Error::InvalidParam);
        }

        let mut transferred: c_int = unsafe { mem::uninitialized() };

        let ptr = buf.as_mut_ptr() as *mut c_uchar;
        let len = buf.len() as c_int;
        let timeout_ms = (timeout.as_secs() * 1000 + timeout.subsec_nanos() as u64 / 1_000_000) as c_uint;

        match unsafe { libusb_bulk_transfer(self.handle, endpoint, ptr, len, &mut transferred, timeout_ms) } {
            0 => {
                Ok(transferred as usize)
            },
            err => {
                if err == LIBUSB_ERROR_INTERRUPTED && transferred > 0 {
                    Ok(transferred as usize)
                }
                else {
                    Err(error::from_libusb(err))
                }
            },
        }
    }

    /// Writes to a bulk endpoint.
    ///
    /// This function attempts to write the contents of `buf` to the bulk endpoint with the address
    /// given by the `endpoint` parameter. The function blocks up to the amount of time specified
    /// by `timeout`.
    ///
    /// If the return value is `Ok(n)`, then `n` bytes of `buf` were written to the endpoint.
    ///
    /// ## Errors
    ///
    /// If this function encounters any form of error while fulfilling the transfer request, an
    /// error variant will be returned. If an error variant is returned, no bytes were written.
    ///
    /// The errors returned by this function include:
    ///
    /// * `InvalidParam` if the endpoint is not an output endpoint.
    /// * `Timeout` if the transfer timed out.
    /// * `Pipe` if the endpoint halted.
    /// * `NoDevice` if the device has been disconnected.
    /// * `Io` if the transfer encountered an I/O error.
    pub fn write_bulk(&self, endpoint: u8, buf: &[u8], timeout: Duration) -> ::Result<usize> {
        if endpoint & LIBUSB_ENDPOINT_DIR_MASK != LIBUSB_ENDPOINT_OUT {
            return Err(Error::InvalidParam);
        }

        let mut transferred: c_int = unsafe { mem::uninitialized() };

        let ptr = buf.as_ptr() as *mut c_uchar;
        let len = buf.len() as c_int;
        let timeout_ms = (timeout.as_secs() * 1000 + timeout.subsec_nanos() as u64 / 1_000_000) as c_uint;

        match unsafe { libusb_bulk_transfer(self.handle, endpoint, ptr, len, &mut transferred, timeout_ms) } {
            0 => {
                Ok(transferred as usize)
            },
            err => {
                if err == LIBUSB_ERROR_INTERRUPTED && transferred > 0 {
                    Ok(transferred as usize)
                }
                else {
                    Err(error::from_libusb(err))
                }
            },
        }
    }

    /// Reads data using a control transfer.
    ///
    /// This function attempts to read data from the device using a control transfer and fills
    /// `buf` with any data received during the transfer. The function blocks up to the amount of
    /// time specified by `timeout`.
    ///
    /// The parameters `request_type`, `request`, `value`, and `index` specify the fields of the
    /// control transfer setup packet (`bmRequestType`, `bRequest`, `wValue`, and `wIndex`
    /// respectively). The values for each of these parameters shall be given in host-endian byte
    /// order. The value for the `request_type` parameter can be built with the helper function,
    /// [request_type()](fn.request_type.html). The meaning of the other parameters depends on the
    /// type of control request.
    ///
    /// If the return value is `Ok(n)`, then `buf` is populated with `n` bytes of data.
    ///
    /// ## Errors
    ///
    /// If this function encounters any form of error while fulfilling the transfer request, an
    /// error variant will be returned. If an error variant is returned, no bytes were read.
    ///
    /// The errors returned by this function include:
    ///
    /// * `InvalidParam` if `request_type` does not specify a read transfer.
    /// * `Timeout` if the transfer timed out.
    /// * `Pipe` if the control request was not supported by the device.
    /// * `NoDevice` if the device has been disconnected.
    /// * `Io` if the transfer encountered an I/O error.
    pub fn read_control(&self, request_type: u8, request: u8, value: u16, index: u16, buf: &mut [u8], timeout: Duration) -> ::Result<usize> {
        if request_type & LIBUSB_ENDPOINT_DIR_MASK != LIBUSB_ENDPOINT_IN {
            return Err(Error::InvalidParam);
        }

        let ptr = buf.as_mut_ptr() as *mut c_uchar;
        let len = buf.len() as u16;
        let timeout_ms = (timeout.as_secs() * 1000 + timeout.subsec_nanos() as u64 / 1_000_000) as c_uint;

        let res = unsafe {
            libusb_control_transfer(self.handle, request_type, request, value, index, ptr, len, timeout_ms)
        };

        if res < 0 {
            Err(error::from_libusb(res))
        } else {
            Ok(res as usize)
        }
    }

    /// Writes data using a control transfer.
    ///
    /// This function attempts to write the contents of `buf` to the device using a control
    /// transfer. The function blocks up to the amount of time specified by `timeout`.
    ///
    /// The parameters `request_type`, `request`, `value`, and `index` specify the fields of the
    /// control transfer setup packet (`bmRequestType`, `bRequest`, `wValue`, and `wIndex`
    /// respectively). The values for each of these parameters shall be given in host-endian byte
    /// order. The value for the `request_type` parameter can be built with the helper function,
    /// [request_type()](fn.request_type.html). The meaning of the other parameters depends on the
    /// type of control request.
    ///
    /// If the return value is `Ok(n)`, then `n` bytes of `buf` were transfered.
    ///
    /// ## Errors
    ///
    /// If this function encounters any form of error while fulfilling the transfer request, an
    /// error variant will be returned. If an error variant is returned, no bytes were read.
    ///
    /// The errors returned by this function include:
    ///
    /// * `InvalidParam` if `request_type` does not specify a write transfer.
    /// * `Timeout` if the transfer timed out.
    /// * `Pipe` if the control request was not supported by the device.
    /// * `NoDevice` if the device has been disconnected.
    /// * `Io` if the transfer encountered an I/O error.
    pub fn write_control(&self, request_type: u8, request: u8, value: u16, index: u16, buf: &[u8], timeout: Duration) -> ::Result<usize> {
        if request_type & LIBUSB_ENDPOINT_DIR_MASK != LIBUSB_ENDPOINT_OUT {
            return Err(Error::InvalidParam);
        }

        let ptr = buf.as_ptr() as *mut c_uchar;
        let len = buf.len() as u16;
        let timeout_ms = (timeout.as_secs() * 1000 + timeout.subsec_nanos() as u64 / 1_000_000) as c_uint;

        let res = unsafe {
            libusb_control_transfer(self.handle, request_type, request, value, index, ptr, len, timeout_ms)
        };

        if res < 0 {
            Err(error::from_libusb(res))
        } else {
            Ok(res as usize)
        }
    }

    /// Reads the languages supported by the device's string descriptors.
    ///
    /// This function returns a list of languages that can be used to read the device's string
    /// descriptors.
    pub fn read_languages(&self, timeout: Duration) -> ::Result<Vec<Language>> {
        let mut buf = Vec::<u8>::with_capacity(256);

        let mut buf_slice = unsafe {
            slice::from_raw_parts_mut((&mut buf[..]).as_mut_ptr(), buf.capacity())
        };

        let len = try!(self.read_control(request_type(Direction::In, RequestType::Standard, Recipient::Device),
                                         LIBUSB_REQUEST_GET_DESCRIPTOR,
                                         (LIBUSB_DT_STRING as u16) << 8,
                                         0,
                                         buf_slice,
                                         timeout));

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
    pub fn read_string_descriptor(&self, language: Language, index: u8, timeout: Duration) -> ::Result<String> {
        let mut buf = Vec::<u8>::with_capacity(256);

        let mut buf_slice = unsafe {
            slice::from_raw_parts_mut((&mut buf[..]).as_mut_ptr(), buf.capacity())
        };

        let len = try!(self.read_control(request_type(Direction::In, RequestType::Standard, Recipient::Device),
                                         LIBUSB_REQUEST_GET_DESCRIPTOR,
                                         (LIBUSB_DT_STRING as u16) << 8 | index as u16,
                                         language.lang_id(),
                                         buf_slice,
                                         timeout));

        unsafe {
            buf.set_len(len);
        }

        let utf16: Vec<u16> = buf.chunks(2).skip(1).map(|chunk| {
            chunk[0] as u16 | (chunk[1] as u16) << 8
        }).collect();

        String::from_utf16(&utf16[..]).map_err(|_| Error::Other)
    }

    /// Reads the device's manufacturer string descriptor.
    pub fn read_manufacturer_string(&self, language: Language, device: &DeviceDescriptor, timeout: Duration) -> ::Result<String> {
        match device.manufacturer_string_index() {
            None => Err(Error::InvalidParam),
            Some(n) => self.read_string_descriptor(language, n, timeout)
        }
    }

    /// Reads the device's product string descriptor.
    pub fn read_product_string(&self, language: Language, device: &DeviceDescriptor, timeout: Duration) -> ::Result<String> {
        match device.product_string_index() {
            None => Err(Error::InvalidParam),
            Some(n) => self.read_string_descriptor(language, n, timeout)
        }
    }

    /// Reads the device's serial number string descriptor.
    pub fn read_serial_number_string(&self, language: Language, device: &DeviceDescriptor, timeout: Duration) -> ::Result<String> {
        match device.serial_number_string_index() {
            None => Err(Error::InvalidParam),
            Some(n) => self.read_string_descriptor(language, n, timeout)
        }
    }

    /// Reads the string descriptor for a configuration's description.
    pub fn read_configuration_string(&self, language: Language, configuration: &ConfigDescriptor, timeout: Duration) -> ::Result<String> {
        match configuration.description_string_index() {
            None => Err(Error::InvalidParam),
            Some(n) => self.read_string_descriptor(language, n, timeout)
        }
    }

    /// Reads the string descriptor for a interface's description.
    pub fn read_interface_string(&self, language: Language, interface: &InterfaceDescriptor, timeout: Duration) -> ::Result<String> {
        match interface.description_string_index() {
            None => Err(Error::InvalidParam),
            Some(n) => self.read_string_descriptor(language, n, timeout)
        }
    }
}

#[doc(hidden)]
pub unsafe fn from_libusb<'a>(context: PhantomData<&'a Context>, handle: *mut libusb_device_handle) -> DeviceHandle<'a> {
    DeviceHandle {
        _context: context,
        handle: handle,
        interfaces: BitSet::with_capacity(u8::max_value() as usize + 1),
    }
}
