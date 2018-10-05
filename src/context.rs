use std::marker::PhantomData;
use std::mem;
use std::ptr;
use std::time::Duration;

use libc::{c_int, c_void, timeval};
use crate::libusb::*;

use crate::device::{self, Device};
use crate::device_list::{self, DeviceList};
use crate::device_handle::{self, DeviceHandle};
use crate::error;

#[cfg(windows)] type Seconds = ::libc::c_long;
#[cfg(windows)] type MicroSeconds = ::libc::c_long;

#[cfg(not(windows))] type Seconds = ::libc::time_t;
#[cfg(not(windows))] type MicroSeconds = ::libc::suseconds_t;


/// A `libusb` context.
pub struct Context {
    context: *mut libusb_context,
}

impl Drop for Context {
    /// Closes the `libusb` context.
    fn drop(&mut self) {
        unsafe {
            libusb_exit(self.context);
        }
    }
}

unsafe impl Sync for Context {}
unsafe impl Send for Context {}

pub trait Hotplug {
	fn device_arrived(&mut self, device: Device);
	fn device_left(&mut self, device: Device);
}

pub type Registration = c_int;

impl Context {
    /// Opens a new `libusb` context.
    pub fn new() -> crate::Result<Self> {
        let mut context = unsafe { mem::uninitialized() };

        try_unsafe!(libusb_init(&mut context));

        Ok(Context { context: context })
    }

    /// Sets the log level of a `libusb` context.
    pub fn set_log_level(&mut self, level: LogLevel) {
        unsafe {
            libusb_set_debug(self.context, level.as_c_int());
        }
    }

    pub fn has_capability(&self) -> bool {
        unsafe {
            libusb_has_capability(LIBUSB_CAP_HAS_CAPABILITY) != 0
        }
    }

    /// Tests whether the running `libusb` library supports hotplug.
    pub fn has_hotplug(&self) -> bool {
        unsafe {
            libusb_has_capability(LIBUSB_CAP_HAS_HOTPLUG) != 0
        }
    }

    /// Tests whether the running `libusb` library has HID access.
    pub fn has_hid_access(&self) -> bool {
        unsafe {
            libusb_has_capability(LIBUSB_CAP_HAS_HID_ACCESS) != 0
        }
    }

    /// Tests whether the running `libusb` library supports detaching the kernel driver.
    pub fn supports_detach_kernel_driver(&self) -> bool {
        unsafe {
            libusb_has_capability(LIBUSB_CAP_SUPPORTS_DETACH_KERNEL_DRIVER) != 0
        }
    }

    /// Returns a list of the current USB devices. The context must outlive the device list.
    pub fn devices<'a>(&'a self) -> crate::Result<DeviceList<'a>> {
        let mut list: *const *mut libusb_device = unsafe { mem::uninitialized() };

        let n = unsafe { libusb_get_device_list(self.context, &mut list) };

        if n < 0 {
            Err(error::from_libusb(n as c_int))
        }
        else {
            Ok(unsafe { device_list::from_libusb(self, list, n as usize) })
        }
    }

    /// Convenience function to open a device by its vendor ID and product ID.
    ///
    /// This function is provided as a convenience for building prototypes without having to
    /// iterate a [`DeviceList`](struct.DeviceList.html). It is not meant for production
    /// applications.
    ///
    /// Returns a device handle for the first device found matching `vendor_id` and `product_id`.
    /// On error, or if the device could not be found, it returns `None`.
    pub fn open_device_with_vid_pid<'a>(&'a self, vendor_id: u16, product_id: u16) -> Option<DeviceHandle<'a>> {
        let handle = unsafe { libusb_open_device_with_vid_pid(self.context, vendor_id, product_id) };

        if handle.is_null() {
            None
        }
        else {
            Some(unsafe { device_handle::from_libusb(PhantomData, handle) })
        }
    }

	pub fn register_callback(&self, vendor_id: Option<u16>, product_id: Option<u16>, class: Option<u8>, callback: Box<Hotplug>) -> crate::Result<Registration> {
		let mut handle: libusb_hotplug_callback_handle = 0;
		let to = Box::new(callback);
		let n = unsafe { libusb_hotplug_register_callback(
			self.context,
			LIBUSB_HOTPLUG_EVENT_DEVICE_ARRIVED | LIBUSB_HOTPLUG_EVENT_DEVICE_LEFT,
			LIBUSB_HOTPLUG_NO_FLAGS,
			vendor_id.map(|v| v as c_int).unwrap_or(LIBUSB_HOTPLUG_MATCH_ANY),
			product_id.map(|v| v as c_int).unwrap_or(LIBUSB_HOTPLUG_MATCH_ANY),
			class.map(|v| v as c_int).unwrap_or(LIBUSB_HOTPLUG_MATCH_ANY),
			hotplug_callback,
			Box::into_raw(to) as *mut c_void,
			&mut handle
		)};
        if n < 0 {
            Err(error::from_libusb(n as c_int))
        }
        else {
            Ok(handle)
        }
	}

	pub fn unregister_callback(&self, reg: Registration) {
		// TODO: fix handler leak
		unsafe { libusb_hotplug_deregister_callback(self.context, reg) }
	}

	pub fn handle_events(&self, timeout: Option<Duration>) -> crate::Result<()>{
		let n = unsafe {
			match timeout {
				Some(t) => {
					let tv = timeval {
						tv_sec: t.as_secs() as Seconds,
						tv_usec: t.subsec_nanos() as MicroSeconds / 1000,
					};
					libusb_handle_events_timeout_completed(self.context, &tv, ptr::null_mut())
				},
				None => libusb_handle_events_completed(self.context, ptr::null_mut())
			}
		};
		if n < 0 {
			Err(error::from_libusb(n as c_int))
		}
		else {
			Ok(())
		}
	}
}

extern "C" fn hotplug_callback(_ctx: *mut libusb_context, device: *mut libusb_device, event: libusb_hotplug_event, reg: *mut c_void) -> c_int {
	let ctx = PhantomData::default();
	unsafe {
		let device = device::from_libusb(ctx, device);
		let reg = reg as *mut Box<Hotplug>;
		match event {
			LIBUSB_HOTPLUG_EVENT_DEVICE_ARRIVED => (*reg).device_arrived(device),
			LIBUSB_HOTPLUG_EVENT_DEVICE_LEFT => (*reg).device_left(device),
			_ => (),
		}
	}
	return 0;
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
            LogLevel::None    => LIBUSB_LOG_LEVEL_NONE,
            LogLevel::Error   => LIBUSB_LOG_LEVEL_ERROR,
            LogLevel::Warning => LIBUSB_LOG_LEVEL_WARNING,
            LogLevel::Info    => LIBUSB_LOG_LEVEL_INFO,
            LogLevel::Debug   => LIBUSB_LOG_LEVEL_DEBUG,
        }
    }
}
