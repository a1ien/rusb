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
