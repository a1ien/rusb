use libc::{c_int, c_void, timeval};

use std::{mem, ptr, sync::Arc, sync::Once, time::Duration};

use crate::{device::Device, device_handle::DeviceHandle, device_list::DeviceList, error};
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
#[derive(Clone, Eq, PartialEq)]
pub struct Context {
    context: Arc<ContextInner>,
}

#[derive(Eq, PartialEq)]
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

pub trait Hotplug<T: UsbContext>: Send {
    fn device_arrived(&mut self, device: Device<T>);
    fn device_left(&mut self, device: Device<T>);
}

#[derive(Debug)]
pub struct Registration<T: UsbContext> {
    context: T,
    handle: libusb_hotplug_callback_handle,
}

impl<T: UsbContext> Registration<T> {
    fn get_handle(&self) -> libusb_hotplug_callback_handle {
        self.handle
    }
}

impl<T: UsbContext> Drop for Registration<T> {
    fn drop(&mut self) {
        let _call_back: Box<CallbackData<T>>;
        #[cfg(libusb_hotplug_get_user_data)]
        unsafe {
            let user_data = libusb_hotplug_get_user_data(self.context.as_raw(), self.get_handle());
            _call_back = Box::<CallbackData<T>>::from_raw(user_data as _);
        }
        unsafe { libusb_hotplug_deregister_callback(self.context.as_raw(), self.get_handle()) }
    }
}

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

    /// Sets the log level of a `libusb` for context.
    fn set_log_level(&mut self, level: LogLevel) {
        unsafe {
            libusb_set_debug(self.as_raw(), level.as_c_int());
        }
    }

    fn register_callback(
        &self,
        vendor_id: Option<u16>,
        product_id: Option<u16>,
        class: Option<u8>,
        enumerate: bool,
        callback: Box<dyn Hotplug<Self>>,
    ) -> crate::Result<Registration<Self>> {
        let mut handle: libusb_hotplug_callback_handle = 0;
        let callback = CallbackData {
            context: self.clone(),
            hotplug: callback,
        };

        let hotplug_flags = if enumerate {
            LIBUSB_HOTPLUG_ENUMERATE
        } else {
            LIBUSB_HOTPLUG_NO_FLAGS
        };

        let to = Box::new(callback);

        let n = unsafe {
            libusb_hotplug_register_callback(
                self.as_raw(),
                LIBUSB_HOTPLUG_EVENT_DEVICE_ARRIVED | LIBUSB_HOTPLUG_EVENT_DEVICE_LEFT,
                hotplug_flags,
                vendor_id
                    .map(c_int::from)
                    .unwrap_or(LIBUSB_HOTPLUG_MATCH_ANY),
                product_id
                    .map(c_int::from)
                    .unwrap_or(LIBUSB_HOTPLUG_MATCH_ANY),
                class.map(c_int::from).unwrap_or(LIBUSB_HOTPLUG_MATCH_ANY),
                hotplug_callback::<Self>,
                Box::into_raw(to) as *mut c_void,
                &mut handle,
            )
        };
        if n < 0 {
            Err(error::from_libusb(n))
        } else {
            Ok(Registration {
                context: self.clone(),
                handle,
            })
        }
    }

    fn unregister_callback(&self, _reg: Registration<Self>) {}

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

struct CallbackData<T: UsbContext> {
    context: T,
    hotplug: Box<dyn Hotplug<T>>,
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

extern "system" fn hotplug_callback<T: UsbContext>(
    _ctx: *mut libusb_context,
    device: *mut libusb_device,
    event: libusb_hotplug_event,
    user_data: *mut c_void,
) -> c_int {
    let ret = std::panic::catch_unwind(|| {
        let reg = unsafe { &mut *(user_data as *mut CallbackData<T>) };
        let device = unsafe {
            Device::from_libusb(
                reg.context.clone(),
                std::ptr::NonNull::new_unchecked(device),
            )
        };
        match event {
            LIBUSB_HOTPLUG_EVENT_DEVICE_ARRIVED => reg.hotplug.device_arrived(device),
            LIBUSB_HOTPLUG_EVENT_DEVICE_LEFT => reg.hotplug.device_left(device),
            _ => (),
        };
    });
    match ret {
        Ok(_) => 0,
        Err(_) => 1,
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
