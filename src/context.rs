use std::mem;

use ::libc::c_int;

use ::device_list::DeviceList;


/// A `libusb` context.
pub struct Context {
    context: *mut ::libusb::libusb_context
}

impl Drop for Context {
    /// Closes the `libusb` context.
    fn drop(&mut self) {
        unsafe {
            ::libusb::libusb_exit(self.context);
        }
    }
}

unsafe impl Sync for Context {}
unsafe impl Send for Context {}

impl Context {
    /// Opens a new `libusb` context.
    pub fn new() -> ::Result<Self> {
        let mut context = unsafe { mem::uninitialized() };

        try_unsafe!(::libusb::libusb_init(&mut context));

        Ok(Context { context: context })
    }

    /// Sets the log level of a `libusb` context.
    pub fn set_log_level(&mut self, level: LogLevel) {
        unsafe {
            ::libusb::libusb_set_debug(self.context, level.as_c_int());
        }
    }

    pub fn has_capability(&self) -> bool {
        unsafe {
            ::libusb::libusb_has_capability(::libusb::LIBUSB_CAP_HAS_CAPABILITY) != 0
        }
    }

    /// Tests whether the running `libusb` library supports hotplug.
    pub fn has_hotplug(&self) -> bool {
        unsafe {
            ::libusb::libusb_has_capability(::libusb::LIBUSB_CAP_HAS_HOTPLUG) != 0
        }
    }

    /// Tests whether the running `libusb` library has HID access.
    pub fn has_hid_access(&self) -> bool {
        unsafe {
            ::libusb::libusb_has_capability(::libusb::LIBUSB_CAP_HAS_HID_ACCESS) != 0
        }
    }

    /// Tests whether the running `libusb` library supports detaching the kernel driver.
    pub fn supports_detach_kernel_driver(&self) -> bool {
        unsafe {
            ::libusb::libusb_has_capability(::libusb::LIBUSB_CAP_SUPPORTS_DETACH_KERNEL_DRIVER) != 0
        }
    }

    /// Returns a list of the current USB devices. The context must outlive the device list.
    pub fn devices<'a>(&'a self) -> ::Result<DeviceList<'a>> {
        let mut list: *const *mut ::libusb::libusb_device = unsafe { mem::uninitialized() };

        let n = unsafe { ::libusb::libusb_get_device_list(self.context, &mut list) };

        if n < 0 {
            Err(::error::from_libusb(n as c_int))
        }
        else {
            Ok(::device_list::from_libusb(self, list, n as usize))
        }
    }
}


/// Library logging levels.
pub enum LogLevel {
    /// No messages are printed by `libusb` (default).
    None,

    /// Error messages printed to `stderr`.
    Error,

    /// Warning and error messages are printed to `stderr`.
    Warning,

    /// Informational messages are printed to `stdout`. Warnings and error messages are printed to
    /// `stderr`.
    Info,

    /// Debug and informational messages are printed to `stdout`. Warnings and error messages are
    /// printed to `stderr`.
    Debug,
}

impl LogLevel {
    fn as_c_int(&self) -> c_int {
        match *self {
            LogLevel::None    => ::libusb::LIBUSB_LOG_LEVEL_NONE,
            LogLevel::Error   => ::libusb::LIBUSB_LOG_LEVEL_ERROR,
            LogLevel::Warning => ::libusb::LIBUSB_LOG_LEVEL_WARNING,
            LogLevel::Info    => ::libusb::LIBUSB_LOG_LEVEL_INFO,
            LogLevel::Debug   => ::libusb::LIBUSB_LOG_LEVEL_DEBUG,
        }
    }
}
