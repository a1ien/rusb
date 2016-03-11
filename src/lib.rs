//! This crate provides a safe wrapper around the native `libusb` library.

pub use libusb_sys::constants;

pub use crate::async_io::{AsyncGroup, Transfer, TransferStatus};
pub use crate::error::{Error, Result};
pub use crate::version::{version, LibraryVersion};

pub use crate::context::{Context, Hotplug, LogLevel, Registration};
pub use crate::device::Device;
pub use crate::device_handle::DeviceHandle;
pub use crate::device_list::{DeviceList, Devices};

pub use crate::config_descriptor::{ConfigDescriptor, Interfaces};
pub use crate::device_descriptor::DeviceDescriptor;
pub use crate::endpoint_descriptor::EndpointDescriptor;
pub use crate::fields::{
    request_type, Direction, Recipient, RequestType, Speed, SyncType, TransferType, UsageType,
    Version,
};
pub use crate::interface_descriptor::{
    EndpointDescriptors, Interface, InterfaceDescriptor, InterfaceDescriptors,
};
pub use crate::language::{Language, PrimaryLanguage, SubLanguage};

#[cfg(test)]
#[macro_use]
mod test_helpers;

#[macro_use]
mod error;
mod version;

mod async_io;
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
