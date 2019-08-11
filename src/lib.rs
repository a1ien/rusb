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
