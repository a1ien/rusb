use crate::{
    constants::{
        LIBUSB_HOTPLUG_ENUMERATE, LIBUSB_HOTPLUG_EVENT_DEVICE_ARRIVED,
        LIBUSB_HOTPLUG_EVENT_DEVICE_LEFT, LIBUSB_HOTPLUG_MATCH_ANY, LIBUSB_HOTPLUG_NO_FLAGS,
    },
    error,
    ffi::{
        libusb_context, libusb_device, libusb_hotplug_callback_handle,
        libusb_hotplug_deregister_callback, libusb_hotplug_event, libusb_hotplug_register_callback,
    },
    Context, Device, Result,
};
use std::{
    borrow::Borrow,
    ffi::c_void,
    fmt::{self, Debug},
    os::raw::c_int,
};

/// When handling a [method@Hotplug::device_arrived] event it is considered safe to call
/// any `rusb` function that takes a [`Device`]. It also safe to open a device and
/// submit **asynchronous** transfers.
/// However, most other functions that take a [`DeviceHandle`] are **not safe** to call.
/// Examples of such functions are any of the synchronous API functions or
/// the blocking functions that retrieve various USB descriptors.
/// These functions must be used outside of the context of the [Hotplug] functions.
///
/// [`Device`]: crate::Device
/// [`DeviceHandle`]: crate::DeviceHandle
/// [`Context::unregister_callback`]: method@crate::Context::unregister_callback
pub trait Hotplug: Send {
    fn device_arrived(&mut self, device: Device);
    fn device_left(&mut self, device: Device);
}

#[derive(Debug)]
#[must_use = "USB hotplug callbacks will be deregistered if the registration is dropped"]
pub struct Registration {
    handle: libusb_hotplug_callback_handle,
    call_back: Box<CallbackData>,
}

impl Registration {
    fn get_handle(&self) -> libusb_hotplug_callback_handle {
        self.handle
    }
}

impl Drop for Registration {
    fn drop(&mut self) {
        unsafe {
            libusb_hotplug_deregister_callback(self.call_back.context.as_raw(), self.get_handle())
        }
    }
}

#[derive(Copy, Clone, Debug, Default)]
#[doc(alias = "libusb_hotplug_register_callback")]
/// Builds hotplug [Registration] with custom configuration values.
pub struct HotplugBuilder {
    vendor_id: Option<u16>,
    product_id: Option<u16>,
    class: Option<u8>,
    enumerate: bool,
}

impl HotplugBuilder {
    /// Returns a new builder with the no filter
    /// Devices can optionally be filtered by [HotplugBuilder::vendor_id]
    /// and [HotplugBuilder::product_id]
    ///
    /// Registration is done by by calling [`register`].
    ///
    /// [`register`]: method@Self::register
    pub fn new() -> Self {
        HotplugBuilder {
            vendor_id: None,
            product_id: None,
            class: None,
            enumerate: false,
        }
    }

    /// Devices can optionally be filtered by vendor
    pub fn vendor_id(&mut self, vendor_id: u16) -> &mut Self {
        self.vendor_id = Some(vendor_id);
        self
    }

    /// Devices can optionally be filtered by product id
    pub fn product_id(&mut self, product_id: u16) -> &mut Self {
        self.product_id = Some(product_id);
        self
    }

    /// Devices can optionally be filtered by class
    pub fn class(&mut self, class: u8) -> &mut Self {
        self.class = Some(class);
        self
    }

    /// If `enumerate` is `true`, then devices that are already
    /// connected will cause your callback's [Hotplug::device_arrived] method to be
    /// called for them.
    pub fn enumerate(&mut self, enumerate: bool) -> &mut Self {
        self.enumerate = enumerate;
        self
    }

    /// Register a `callback` to be called on hotplug events. The callback's
    /// [method@Hotplug::device_arrived] method is called when a new device is added to
    /// the bus, and [method@Hotplug::device_left] is called when it is removed.
    ///
    /// The callback will remain registered until the returned [Registration] is
    /// dropped, which can be done explicitly with [`Context::unregister_callback`].
    ///
    /// When handling a [method@Hotplug::device_arrived] event it is considered safe to call
    /// any `rusb` function that takes a [`Device`]. It also safe to open a device and
    /// submit **asynchronous** transfers.
    /// However, most other functions that take a [`DeviceHandle`] are **not safe** to call.
    /// Examples of such functions are any of the synchronous API functions or
    /// the blocking functions that retrieve various USB descriptors.
    /// These functions must be used outside of the context of the [Hotplug] functions.
    ///
    /// [`Device`]: crate::Device
    /// [`DeviceHandle`]: crate::DeviceHandle
    /// [`Context::unregister_callback`]: method@crate::Context::unregister_callback
    pub fn register(self, context: Context, callback: Box<dyn Hotplug>) -> Result<Registration> {
        let mut handle: libusb_hotplug_callback_handle = 0;
        let mut call_back = Box::new(CallbackData {
            context: context.borrow().clone(),
            hotplug: callback,
        });

        let hotplug_flags = if self.enumerate {
            LIBUSB_HOTPLUG_ENUMERATE
        } else {
            LIBUSB_HOTPLUG_NO_FLAGS
        };

        let user_data = &mut *call_back as *mut _ as *mut _;

        let n = unsafe {
            libusb_hotplug_register_callback(
                context.borrow().as_raw(),
                LIBUSB_HOTPLUG_EVENT_DEVICE_ARRIVED | LIBUSB_HOTPLUG_EVENT_DEVICE_LEFT,
                hotplug_flags,
                self.vendor_id
                    .map(c_int::from)
                    .unwrap_or(LIBUSB_HOTPLUG_MATCH_ANY),
                self.product_id
                    .map(c_int::from)
                    .unwrap_or(LIBUSB_HOTPLUG_MATCH_ANY),
                self.class
                    .map(c_int::from)
                    .unwrap_or(LIBUSB_HOTPLUG_MATCH_ANY),
                hotplug_callback,
                user_data,
                &mut handle,
            )
        };
        if n < 0 {
            Err(error::from_libusb(n))
        } else {
            Ok(Registration { handle, call_back })
        }
    }
}

struct CallbackData {
    context: Context,
    hotplug: Box<dyn Hotplug>,
}

impl Debug for CallbackData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("CallbackData")
            .field("context", &self.context)
            .finish()
    }
}

pub extern "system" fn hotplug_callback(
    _ctx: *mut libusb_context,
    device: *mut libusb_device,
    event: libusb_hotplug_event,
    user_data: *mut c_void,
) -> c_int {
    let ret = std::panic::catch_unwind(|| {
        let reg = unsafe { &mut *(user_data as *mut CallbackData) };
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
