//! This crate provides a safe wrapper around the native `libusb` library.

pub use libusb1_sys::constants;

pub use crate::{
    config_descriptor::{ConfigDescriptor, Interfaces},
    context::{Context, GlobalContext, Hotplug, LogLevel, Registration, UsbContext},
    device::Device,
    device_descriptor::DeviceDescriptor,
    device_handle::DeviceHandle,
    device_list::{DeviceList, Devices},
    endpoint_descriptor::EndpointDescriptor,
    error::{Error, Result},
    fields::{
        request_type, Direction, Recipient, RequestType, Speed, SyncType, TransferType, UsageType,
        Version,
    },
    interface_descriptor::{
        EndpointDescriptors, Interface, InterfaceDescriptor, InterfaceDescriptors,
    },
    language::{Language, PrimaryLanguage, SubLanguage},
    options::UsbOption,
    version::{version, LibraryVersion},
};

#[cfg(test)]
#[macro_use]
mod test_helpers;

#[macro_use]
mod error;
mod version;

mod context;
mod device;
mod device_handle;
mod device_list;

mod config_descriptor;
mod device_descriptor;
mod endpoint_descriptor;
mod fields;
mod interface_descriptor;
mod language;
mod options;

/// Tests whether the running `libusb` library supports capability API.
pub fn has_capability() -> bool {
    GlobalContext::default().as_raw();
    unsafe { libusb1_sys::libusb_has_capability(constants::LIBUSB_CAP_HAS_CAPABILITY) != 0 }
}

/// Tests whether the running `libusb` library supports hotplug.
pub fn has_hotplug() -> bool {
    GlobalContext::default().as_raw();
    unsafe { libusb1_sys::libusb_has_capability(constants::LIBUSB_CAP_HAS_HOTPLUG) != 0 }
}

/// Tests whether the running `libusb` library has HID access.
pub fn has_hid_access() -> bool {
    GlobalContext::default().as_raw();
    unsafe { libusb1_sys::libusb_has_capability(constants::LIBUSB_CAP_HAS_HID_ACCESS) != 0 }
}

/// Tests whether the running `libusb` library supports detaching the kernel driver.
pub fn supports_detach_kernel_driver() -> bool {
    GlobalContext::default().as_raw();
    unsafe {
        libusb1_sys::libusb_has_capability(constants::LIBUSB_CAP_SUPPORTS_DETACH_KERNEL_DRIVER) != 0
    }
}

/// Returns a list of the current USB devices. Using global context
pub fn devices() -> crate::Result<DeviceList<GlobalContext>> {
    GlobalContext::default().devices()
}

/// Sets the log level of a `libusb` global context.
pub fn set_log_level(level: LogLevel) {
    unsafe {
        libusb1_sys::libusb_set_debug(GlobalContext::default().as_raw(), level.as_c_int());
    }
}

/// Convenience function to open a device by its vendor ID and product ID.
/// Using global context
///
/// This function is provided as a convenience for building prototypes without having to
/// iterate a [`DeviceList`](struct.DeviceList.html). It is not meant for production
/// applications.
///
/// Returns a device handle for the first device found matching `vendor_id` and `product_id`.
/// On error, or if the device could not be found, it returns `None`.
pub fn open_device_with_vid_pid(
    vendor_id: u16,
    product_id: u16,
) -> Option<DeviceHandle<GlobalContext>> {
    let handle = unsafe {
        libusb1_sys::libusb_open_device_with_vid_pid(
            GlobalContext::default().as_raw(),
            vendor_id,
            product_id,
        )
    };

    if handle.is_null() {
        None
    } else {
        Some(unsafe {
            DeviceHandle::from_libusb(
                GlobalContext::default(),
                std::ptr::NonNull::new_unchecked(handle),
            )
        })
    }
}
