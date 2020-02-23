use std::{mem, ptr::NonNull, slice, time::Duration};

use bit_set::BitSet;
use libc::{c_int, c_uchar, c_uint};
use libusb1_sys::{constants::*, *};

use crate::{
    config_descriptor::ConfigDescriptor,
    device::{self, Device},
    device_descriptor::DeviceDescriptor,
    error::{self, Error},
    fields::{request_type, Direction, Recipient, RequestType},
    interface_descriptor::InterfaceDescriptor,
    language::Language,
    UsbContext,
};

/// A handle to an open USB device.
#[derive(Eq, PartialEq)]
pub struct DeviceHandle<T: UsbContext> {
    context: T,
    handle: NonNull<libusb_device_handle>,
    interfaces: BitSet,
}

impl<T: UsbContext> Drop for DeviceHandle<T> {
    /// Closes the device.
    fn drop(&mut self) {
        unsafe {
            for iface in self.interfaces.iter() {
                libusb_release_interface(self.handle.as_ptr(), iface as c_int);
            }

            libusb_close(self.handle.as_ptr());
        }
    }
}

unsafe impl<T: UsbContext> Send for DeviceHandle<T> {}
unsafe impl<T: UsbContext> Sync for DeviceHandle<T> {}

impl<T: UsbContext> DeviceHandle<T> {
    /// Get the raw libusb_device_handle pointer, for advanced use in unsafe code.
    ///
    /// This structure tracks claimed interfaces, and will get out if sync if interfaces are
    /// manipulated externally. Use only libusb endpoint IO functions.
    pub fn as_raw(&self) -> *mut libusb_device_handle {
        self.handle.as_ptr()
    }

    /// Get the device associated to this handle
    pub fn device(&self) -> Device<T> {
        unsafe {
            device::from_libusb(
                self.context.clone(),
                libusb_get_device(self.handle.as_ptr()),
            )
        }
    }

    /// Returns the active configuration number.
    pub fn active_configuration(&self) -> crate::Result<u8> {
        let mut config = mem::MaybeUninit::<c_int>::uninit();

        try_unsafe!(libusb_get_configuration(
            self.handle.as_ptr(),
            config.as_mut_ptr()
        ));
        Ok(unsafe { config.assume_init() } as u8)
    }

    /// Sets the device's active configuration.
    pub fn set_active_configuration(&mut self, config: u8) -> crate::Result<()> {
        try_unsafe!(libusb_set_configuration(
            self.handle.as_ptr(),
            c_int::from(config)
        ));
        Ok(())
    }

    /// Puts the device in an unconfigured state.
    pub fn unconfigure(&mut self) -> crate::Result<()> {
        try_unsafe!(libusb_set_configuration(self.handle.as_ptr(), -1));
        Ok(())
    }

    /// Resets the device.
    pub fn reset(&mut self) -> crate::Result<()> {
        try_unsafe!(libusb_reset_device(self.handle.as_ptr()));
        Ok(())
    }

    /// Clear the halt/stall condition for an endpoint.
    pub fn clear_halt(&mut self, endpoint: u8) -> crate::Result<()> {
        try_unsafe!(libusb_clear_halt(self.handle.as_ptr(), endpoint));
        Ok(())
    }

    /// Indicates whether the device has an attached kernel driver.
    ///
    /// This method is not supported on all platforms.
    pub fn kernel_driver_active(&self, iface: u8) -> crate::Result<bool> {
        match unsafe { libusb_kernel_driver_active(self.handle.as_ptr(), c_int::from(iface)) } {
            0 => Ok(false),
            1 => Ok(true),
            err => Err(error::from_libusb(err)),
        }
    }

    /// Detaches an attached kernel driver from the device.
    ///
    /// This method is not supported on all platforms.
    pub fn detach_kernel_driver(&mut self, iface: u8) -> crate::Result<()> {
        try_unsafe!(libusb_detach_kernel_driver(
            self.handle.as_ptr(),
            c_int::from(iface)
        ));
        Ok(())
    }

    /// Attaches a kernel driver to the device.
    ///
    /// This method is not supported on all platforms.
    pub fn attach_kernel_driver(&mut self, iface: u8) -> crate::Result<()> {
        try_unsafe!(libusb_attach_kernel_driver(
            self.handle.as_ptr(),
            c_int::from(iface)
        ));
        Ok(())
    }

    /// Enable/disable automatic kernel driver detachment.
    ///
    /// When this is enabled rusb will automatically detach the
    /// kernel driver on an interface when claiming the interface, and
    /// attach it when releasing the interface.
    ///
    /// On platforms which do not have support, this function will
    /// return `Error::NotSupported`, and rusb will continue as if
    /// this function was never called.
    pub fn set_auto_detach_kernel_driver(&mut self, auto_detach: bool) -> crate::Result<()> {
        try_unsafe!(libusb_set_auto_detach_kernel_driver(
            self.handle.as_ptr(),
            auto_detach.into()
        ));
        Ok(())
    }

    /// Claims one of the device's interfaces.
    ///
    /// An interface must be claimed before operating on it. All claimed interfaces are released
    /// when the device handle goes out of scope.
    pub fn claim_interface(&mut self, iface: u8) -> crate::Result<()> {
        try_unsafe!(libusb_claim_interface(
            self.handle.as_ptr(),
            c_int::from(iface)
        ));
        self.interfaces.insert(iface as usize);
        Ok(())
    }

    /// Releases a claimed interface.
    pub fn release_interface(&mut self, iface: u8) -> crate::Result<()> {
        try_unsafe!(libusb_release_interface(
            self.handle.as_ptr(),
            c_int::from(iface)
        ));
        self.interfaces.remove(iface as usize);
        Ok(())
    }

    /// Sets an interface's active setting.
    pub fn set_alternate_setting(&mut self, iface: u8, setting: u8) -> crate::Result<()> {
        try_unsafe!(libusb_set_interface_alt_setting(
            self.handle.as_ptr(),
            c_int::from(iface),
            c_int::from(setting)
        ));
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
    pub fn read_interrupt(
        &self,
        endpoint: u8,
        buf: &mut [u8],
        timeout: Duration,
    ) -> crate::Result<usize> {
        if endpoint & LIBUSB_ENDPOINT_DIR_MASK != LIBUSB_ENDPOINT_IN {
            return Err(Error::InvalidParam);
        }
        let mut transferred = mem::MaybeUninit::<c_int>::uninit();
        unsafe {
            match libusb_interrupt_transfer(
                self.handle.as_ptr(),
                endpoint,
                buf.as_mut_ptr() as *mut c_uchar,
                buf.len() as c_int,
                transferred.as_mut_ptr(),
                timeout.as_millis() as c_uint,
            ) {
                0 => Ok(transferred.assume_init() as usize),
                err if err == LIBUSB_ERROR_INTERRUPTED => {
                    let transferred = transferred.assume_init();
                    if transferred > 0 {
                        Ok(transferred as usize)
                    } else {
                        Err(error::from_libusb(err))
                    }
                }
                err => Err(error::from_libusb(err)),
            }
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
    pub fn write_interrupt(
        &self,
        endpoint: u8,
        buf: &[u8],
        timeout: Duration,
    ) -> crate::Result<usize> {
        if endpoint & LIBUSB_ENDPOINT_DIR_MASK != LIBUSB_ENDPOINT_OUT {
            return Err(Error::InvalidParam);
        }
        let mut transferred = mem::MaybeUninit::<c_int>::uninit();
        unsafe {
            match libusb_interrupt_transfer(
                self.handle.as_ptr(),
                endpoint,
                buf.as_ptr() as *mut c_uchar,
                buf.len() as c_int,
                transferred.as_mut_ptr(),
                timeout.as_millis() as c_uint,
            ) {
                0 => Ok(transferred.assume_init() as usize),
                err if err == LIBUSB_ERROR_INTERRUPTED => {
                    let transferred = transferred.assume_init();
                    if transferred > 0 {
                        Ok(transferred as usize)
                    } else {
                        Err(error::from_libusb(err))
                    }
                }
                err => Err(error::from_libusb(err)),
            }
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
    pub fn read_bulk(
        &self,
        endpoint: u8,
        buf: &mut [u8],
        timeout: Duration,
    ) -> crate::Result<usize> {
        if endpoint & LIBUSB_ENDPOINT_DIR_MASK != LIBUSB_ENDPOINT_IN {
            return Err(Error::InvalidParam);
        }
        let mut transferred = mem::MaybeUninit::<c_int>::uninit();
        unsafe {
            match libusb_bulk_transfer(
                self.handle.as_ptr(),
                endpoint,
                buf.as_mut_ptr() as *mut c_uchar,
                buf.len() as c_int,
                transferred.as_mut_ptr(),
                timeout.as_millis() as c_uint,
            ) {
                0 => Ok(transferred.assume_init() as usize),
                err if err == LIBUSB_ERROR_INTERRUPTED || err == LIBUSB_ERROR_TIMEOUT => {
                    let transferred = transferred.assume_init();
                    if transferred > 0 {
                        Ok(transferred as usize)
                    } else {
                        Err(error::from_libusb(err))
                    }
                }
                err => Err(error::from_libusb(err)),
            }
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
    pub fn write_bulk(&self, endpoint: u8, buf: &[u8], timeout: Duration) -> crate::Result<usize> {
        if endpoint & LIBUSB_ENDPOINT_DIR_MASK != LIBUSB_ENDPOINT_OUT {
            return Err(Error::InvalidParam);
        }
        let mut transferred = mem::MaybeUninit::<c_int>::uninit();
        unsafe {
            match libusb_bulk_transfer(
                self.handle.as_ptr(),
                endpoint,
                buf.as_ptr() as *mut c_uchar,
                buf.len() as c_int,
                transferred.as_mut_ptr(),
                timeout.as_millis() as c_uint,
            ) {
                0 => Ok(transferred.assume_init() as usize),
                err if err == LIBUSB_ERROR_INTERRUPTED || err == LIBUSB_ERROR_TIMEOUT => {
                    let transferred = transferred.assume_init();
                    if transferred > 0 {
                        Ok(transferred as usize)
                    } else {
                        Err(error::from_libusb(err))
                    }
                }
                err => Err(error::from_libusb(err)),
            }
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
    pub fn read_control(
        &self,
        request_type: u8,
        request: u8,
        value: u16,
        index: u16,
        buf: &mut [u8],
        timeout: Duration,
    ) -> crate::Result<usize> {
        if request_type & LIBUSB_ENDPOINT_DIR_MASK != LIBUSB_ENDPOINT_IN {
            return Err(Error::InvalidParam);
        }
        let res = unsafe {
            libusb_control_transfer(
                self.handle.as_ptr(),
                request_type,
                request,
                value,
                index,
                buf.as_mut_ptr() as *mut c_uchar,
                buf.len() as u16,
                timeout.as_millis() as c_uint,
            )
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
    pub fn write_control(
        &self,
        request_type: u8,
        request: u8,
        value: u16,
        index: u16,
        buf: &[u8],
        timeout: Duration,
    ) -> crate::Result<usize> {
        if request_type & LIBUSB_ENDPOINT_DIR_MASK != LIBUSB_ENDPOINT_OUT {
            return Err(Error::InvalidParam);
        }
        let res = unsafe {
            libusb_control_transfer(
                self.handle.as_ptr(),
                request_type,
                request,
                value,
                index,
                buf.as_ptr() as *mut c_uchar,
                buf.len() as u16,
                timeout.as_millis() as c_uint,
            )
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
    pub fn read_languages(&self, timeout: Duration) -> crate::Result<Vec<Language>> {
        let mut buf = [0u8; 255];

        let buf_slice = unsafe { slice::from_raw_parts_mut(buf.as_mut_ptr(), buf.len()) };

        let len = self.read_control(
            request_type(Direction::In, RequestType::Standard, Recipient::Device),
            LIBUSB_REQUEST_GET_DESCRIPTOR,
            u16::from(LIBUSB_DT_STRING) << 8,
            0,
            buf_slice,
            timeout,
        )?;

        Ok(buf_slice[0..len]
            .chunks(2)
            .skip(1)
            .map(|chunk| {
                let lang_id = u16::from(chunk[0]) | u16::from(chunk[1]) << 8;
                crate::language::from_lang_id(lang_id)
            })
            .collect())
    }

    /// Reads a ascii string descriptor from the device.
    ///
    pub fn read_string_descriptor_ascii(&self, index: u8) -> crate::Result<String> {
        let mut buf = Vec::<u8>::with_capacity(255);

        let buf_slice = unsafe { slice::from_raw_parts_mut(buf.as_mut_ptr(), buf.capacity()) };

        let ptr = buf_slice.as_mut_ptr() as *mut c_uchar;
        let len = buf_slice.len() as i32;

        let res =
            unsafe { libusb_get_string_descriptor_ascii(self.handle.as_ptr(), index, ptr, len) };

        if res < 0 {
            return Err(error::from_libusb(res));
        }

        unsafe {
            buf.set_len(res as usize);
        }

        String::from_utf8(buf).map_err(|_| Error::Other)
    }

    /// Reads a string descriptor from the device.
    ///
    /// `language` should be one of the languages returned from [`read_languages`](#method.read_languages).
    pub fn read_string_descriptor(
        &self,
        language: Language,
        index: u8,
        timeout: Duration,
    ) -> crate::Result<String> {
        let mut buf = [0u8; 255];

        let len = self.read_control(
            request_type(Direction::In, RequestType::Standard, Recipient::Device),
            LIBUSB_REQUEST_GET_DESCRIPTOR,
            u16::from(LIBUSB_DT_STRING) << 8 | u16::from(index),
            language.lang_id(),
            &mut buf,
            timeout,
        )?;

        if len < 2 || buf[0] != len as u8 || len & 0x01 != 0 {
            // Consider making this `Error::BadDescriptor` on next breaking change.
            return Err(Error::Other);
        }

        if len == 2 {
            return Ok(String::new());
        }

        let utf16: Vec<u16> = buf[..len]
            .chunks(2)
            .skip(1)
            .map(|chunk| u16::from(chunk[0]) | u16::from(chunk[1]) << 8)
            .collect();

        String::from_utf16(&utf16).map_err(|_| Error::Other)
    }

    /// Reads the device's manufacturer string descriptor (ascii).
    pub fn read_manufacturer_string_ascii(
        &self,
        device: &DeviceDescriptor,
    ) -> crate::Result<String> {
        match device.manufacturer_string_index() {
            None => Err(Error::InvalidParam),
            Some(n) => self.read_string_descriptor_ascii(n),
        }
    }

    /// Reads the device's manufacturer string descriptor.
    pub fn read_manufacturer_string(
        &self,
        language: Language,
        device: &DeviceDescriptor,
        timeout: Duration,
    ) -> crate::Result<String> {
        match device.manufacturer_string_index() {
            None => Err(Error::InvalidParam),
            Some(n) => self.read_string_descriptor(language, n, timeout),
        }
    }

    /// Reads the device's product string descriptor (ascii).
    pub fn read_product_string_ascii(&self, device: &DeviceDescriptor) -> crate::Result<String> {
        match device.product_string_index() {
            None => Err(Error::InvalidParam),
            Some(n) => self.read_string_descriptor_ascii(n),
        }
    }

    /// Reads the device's product string descriptor.
    pub fn read_product_string(
        &self,
        language: Language,
        device: &DeviceDescriptor,
        timeout: Duration,
    ) -> crate::Result<String> {
        match device.product_string_index() {
            None => Err(Error::InvalidParam),
            Some(n) => self.read_string_descriptor(language, n, timeout),
        }
    }

    /// Reads the device's serial number string descriptor (ascii).
    pub fn read_serial_number_string_ascii(
        &self,
        device: &DeviceDescriptor,
    ) -> crate::Result<String> {
        match device.serial_number_string_index() {
            None => Err(Error::InvalidParam),
            Some(n) => self.read_string_descriptor_ascii(n),
        }
    }

    /// Reads the device's serial number string descriptor.
    pub fn read_serial_number_string(
        &self,
        language: Language,
        device: &DeviceDescriptor,
        timeout: Duration,
    ) -> crate::Result<String> {
        match device.serial_number_string_index() {
            None => Err(Error::InvalidParam),
            Some(n) => self.read_string_descriptor(language, n, timeout),
        }
    }

    /// Reads the string descriptor for a configuration's description.
    pub fn read_configuration_string(
        &self,
        language: Language,
        configuration: &ConfigDescriptor,
        timeout: Duration,
    ) -> crate::Result<String> {
        match configuration.description_string_index() {
            None => Err(Error::InvalidParam),
            Some(n) => self.read_string_descriptor(language, n, timeout),
        }
    }

    /// Reads the string descriptor for a interface's description.
    pub fn read_interface_string(
        &self,
        language: Language,
        interface: &InterfaceDescriptor,
        timeout: Duration,
    ) -> crate::Result<String> {
        match interface.description_string_index() {
            None => Err(Error::InvalidParam),
            Some(n) => self.read_string_descriptor(language, n, timeout),
        }
    }
}

#[doc(hidden)]
pub(crate) unsafe fn from_libusb<T: UsbContext>(
    context: T,
    handle: *mut libusb_device_handle,
) -> DeviceHandle<T> {
    DeviceHandle {
        context: context,
        handle: NonNull::new_unchecked(handle),
        interfaces: BitSet::with_capacity(u8::max_value() as usize + 1),
    }
}
