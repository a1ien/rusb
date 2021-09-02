use libc::{c_int, timeval};

use std::{mem, ptr, sync::Arc, sync::Once, time::Duration};

#[cfg(unix)]
use std::os::unix::io::RawFd;

use crate::hotplug::{Hotplug, HotplugBuilder, Registration};
use crate::{device_handle::DeviceHandle, device_list::DeviceList, error};
use libusb1_sys::{constants::*, *};

#[cfg(windows)]
type Seconds = ::libc::c_long;
#[cfg(windows)]
type MicroSeconds = ::libc::c_long;

#[cfg(not(windows))]
type Seconds = ::libc::time_t;
#[cfg(not(windows))]
type MicroSeconds = ::libc::suseconds_t;

#[derive(Copy, Clone, Eq, PartialEq, Default)]
pub struct GlobalContext {}

/// A `libusb` context.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Context {
    context: Arc<ContextInner>,
}

#[derive(Debug, Eq, PartialEq)]
struct ContextInner {
    inner: ptr::NonNull<libusb_context>,
}

impl Drop for ContextInner {
    /// Closes the `libusb` context.
    fn drop(&mut self) {
        unsafe {
            libusb_exit(self.inner.as_ptr());
        }
    }
}

unsafe impl Sync for Context {}
unsafe impl Send for Context {}

pub trait UsbContext: Clone + Sized + Send + Sync {
    /// Get the raw libusb_context pointer, for advanced use in unsafe code.
    fn as_raw(&self) -> *mut libusb_context;

    /// Returns a list of the current USB devices.
    fn devices(&self) -> crate::Result<DeviceList<Self>> {
        DeviceList::new_with_context(self.clone())
    }

    /// Convenience function to open a device by its vendor ID and product ID.
    ///
    /// This function is provided as a convenience for building prototypes without having to
    /// iterate a [`DeviceList`](struct.DeviceList.html). It is not meant for production
    /// applications.
    ///
    /// Returns a device handle for the first device found matching `vendor_id` and `product_id`.
    /// On error, or if the device could not be found, it returns `None`.
    fn open_device_with_vid_pid(
        &self,
        vendor_id: u16,
        product_id: u16,
    ) -> Option<DeviceHandle<Self>> {
        let handle =
            unsafe { libusb_open_device_with_vid_pid(self.as_raw(), vendor_id, product_id) };
        let ptr = std::ptr::NonNull::new(handle)?;
        Some(unsafe { DeviceHandle::from_libusb(self.clone(), ptr) })
    }

    /// Opens the device with a pre-opened file descriptor.
    ///
    /// This is UNIX-only and platform-specific. It is currently working with
    /// Linux/Android, but might work with other systems in the future.
    ///
    /// Note: This function does not take ownership of the specified file
    /// descriptor. The caller has the responsibility of keeping it opened for
    /// as long as the device handle.
    #[cfg(unix)]
    #[doc(alias = "libusb_wrap_sys_device")]
    unsafe fn open_device_with_fd(&self, fd: RawFd) -> crate::Result<DeviceHandle<Self>> {
        let mut handle = mem::MaybeUninit::<*mut libusb_device_handle>::uninit();

        match libusb_wrap_sys_device(self.as_raw(), fd as _, handle.as_mut_ptr()) {
            0 => {
                let ptr =
                    std::ptr::NonNull::new(handle.assume_init()).ok_or(crate::Error::NoDevice)?;

                Ok(DeviceHandle::from_libusb(self.clone(), ptr))
            }
            err => Err(error::from_libusb(err)),
        }
    }

    /// Sets the log level of a `libusb` for context.
    fn set_log_level(&mut self, level: LogLevel) {
        unsafe {
            libusb_set_debug(self.as_raw(), level.as_c_int());
        }
    }

    /// Register a callback to be called on hotplug events. The callback's
    /// [Hotplug::device_arrived] method is called when a new device is added to
    /// the bus, and [Hotplug::device_left] is called when it is removed.
    ///
    /// Devices can optionally be filtered by vendor (`vendor_id`) and device id
    /// (`product_id`).
    ///
    /// The callback will remain registered until the returned [Registration] is
    /// dropped, which can be done explicitly with [Context::unregister_callback].
    ///
    /// When handling a [Hotplug::device_arrived] event it is considered safe to call
    /// any `rusb` function that takes a [crate::Device]. It also safe to open a device and
    /// submit **asynchronous** transfers.
    /// However, most other functions that take a [DeviceHandle] are **not safe** to call.
    /// Examples of such functions are any of the synchronous API functions or
    /// the blocking functions that retrieve various USB descriptors.
    /// These functions must be used outside of the context of the [Hotplug] functions.
    #[deprecated(since = "0.9.0", note = "Use HotplugBuilder")]
    fn register_callback(
        &self,
        vendor_id: Option<u16>,
        product_id: Option<u16>,
        class: Option<u8>,
        callback: Box<dyn Hotplug<Self>>,
    ) -> crate::Result<Registration<Self>> {
        let mut builder = HotplugBuilder::new();

        let mut builder = &mut builder;
        if let Some(vendor_id) = vendor_id {
            builder = builder.vendor_id(vendor_id)
        }
        if let Some(product_id) = product_id {
            builder = builder.product_id(product_id)
        }
        if let Some(class) = class {
            builder = builder.class(class)
        }

        builder.register(self, callback)
    }

    /// Unregisters the callback corresponding to the given registration. The
    /// same thing can be achieved by dropping the registration.
    fn unregister_callback(&self, _reg: Registration<Self>) {}

    /// Handle any pending events.
    /// If timeout less then 1 microseconds then this function will handle any already-pending
    /// events and then immediately return in non-blocking style.
    /// If timeout is [None] then function will handle any pending events in blocking mode.
    fn handle_events(&self, timeout: Option<Duration>) -> crate::Result<()> {
        let n = unsafe {
            match timeout {
                Some(t) => {
                    let tv = timeval {
                        tv_sec: t.as_secs() as Seconds,
                        tv_usec: t.subsec_nanos() as MicroSeconds / 1000,
                    };
                    libusb_handle_events_timeout_completed(self.as_raw(), &tv, ptr::null_mut())
                }
                None => libusb_handle_events_completed(self.as_raw(), ptr::null_mut()),
            }
        };
        if n < 0 {
            Err(error::from_libusb(n as c_int))
        } else {
            Ok(())
        }
    }

    /// Interrupt any active thread that is handling events (for example with
    /// [handle_events][`Self::handle_events()`]).
    #[doc(alias = "libusb_interrupt_event_handler")]
    fn interrupt_handle_events(&self) {
        unsafe { libusb_interrupt_event_handler(self.as_raw()) }
    }

    fn next_timeout(&self) -> crate::Result<Option<Duration>> {
        let mut tv = timeval {
            tv_sec: 0,
            tv_usec: 0,
        };
        let n = unsafe { libusb_get_next_timeout(self.as_raw(), &mut tv) };

        if n < 0 {
            Err(error::from_libusb(n as c_int))
        } else if n == 0 {
            Ok(None)
        } else {
            let duration = Duration::new(tv.tv_sec as _, (tv.tv_usec * 1000) as _);
            Ok(Some(duration))
        }
    }
}

impl UsbContext for Context {
    fn as_raw(&self) -> *mut libusb_context {
        self.context.inner.as_ptr()
    }
}

impl UsbContext for GlobalContext {
    fn as_raw(&self) -> *mut libusb_context {
        static mut USB_CONTEXT: *mut libusb_context = ptr::null_mut();
        static ONCE: Once = Once::new();

        ONCE.call_once(|| {
            let mut context = mem::MaybeUninit::<*mut libusb_context>::uninit();
            unsafe {
                USB_CONTEXT = match libusb_init(context.as_mut_ptr()) {
                    0 => context.assume_init(),
                    err => panic!(
                        "Can't init Global usb context, error {:?}",
                        error::from_libusb(err)
                    ),
                }
            };
        });
        // Clone data that is safe to use concurrently.
        unsafe { USB_CONTEXT }
    }
}

impl Context {
    /// Opens a new `libusb` context.
    pub fn new() -> crate::Result<Self> {
        let mut context = mem::MaybeUninit::<*mut libusb_context>::uninit();

        try_unsafe!(libusb_init(context.as_mut_ptr()));

        Ok(Context {
            context: unsafe {
                Arc::new(ContextInner {
                    inner: ptr::NonNull::new_unchecked(context.assume_init()),
                })
            },
        })
    }

    /// Creates a new `libusb` context and sets runtime options.
    pub fn with_options(opts: &[crate::UsbOption]) -> crate::Result<Self> {
        let mut this = Self::new()?;

        for opt in opts {
            opt.apply(&mut this)?;
        }

        Ok(this)
    }
}

/// Library logging levels.
#[derive(Clone, Copy)]
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
    pub(crate) fn as_c_int(self) -> c_int {
        match self {
            LogLevel::None => LIBUSB_LOG_LEVEL_NONE,
            LogLevel::Error => LIBUSB_LOG_LEVEL_ERROR,
            LogLevel::Warning => LIBUSB_LOG_LEVEL_WARNING,
            LogLevel::Info => LIBUSB_LOG_LEVEL_INFO,
            LogLevel::Debug => LIBUSB_LOG_LEVEL_DEBUG,
        }
    }
}
