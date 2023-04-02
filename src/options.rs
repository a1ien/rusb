use crate::{error, UsbContext};
use libusb1_sys::{constants::*, libusb_set_option};

/// A `libusb` runtime option that can be enabled for a context.
pub struct UsbOption {
    inner: OptionInner,
}

impl UsbOption {
    /// Use the [UsbDk] backend if available.
    ///
    /// **Note**: This method is available on **Windows** only!
    ///
    /// [UsbDk]: https://github.com/daynix/UsbDk
    #[cfg(windows)]
    pub fn use_usbdk() -> Self {
        Self {
            inner: OptionInner::UseUsbdk,
        }
    }

    pub(crate) fn apply<T: UsbContext>(&self, ctx: &mut T) -> crate::Result<()> {
        match self.inner {
            OptionInner::UseUsbdk => {
                let err = unsafe { libusb_set_option(ctx.as_raw(), LIBUSB_OPTION_USE_USBDK) };
                if err == LIBUSB_SUCCESS {
                    Ok(())
                } else {
                    Err(error::from_libusb(err))
                }
            }
        }
    }
}

enum OptionInner {
    #[cfg_attr(not(windows), allow(dead_code))] // only constructed on Windows
    UseUsbdk,
}

/// Disable device scanning in `libusb` init.
///
/// Hotplug functionality will also be deactivated.
///
/// This is a Linux only option and it must be set before any [`Context`]
/// creation.
///
/// The option is useful in combination with [`Context::open_device_with_fd()`],
/// which can access a device directly without prior device scanning.
#[cfg(unix)]
pub fn disable_device_discovery() -> crate::Result<()> {
    try_unsafe!(libusb1_sys::libusb_set_option(
        std::ptr::null_mut(),
        LIBUSB_OPTION_NO_DEVICE_DISCOVERY
    ));
    Ok(())
}
