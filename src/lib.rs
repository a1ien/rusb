//! This crate provides a safe wrapper around the native `libusb` library.

extern crate bit_set;
extern crate libusb_sys as libusb;
extern crate libc;

pub use crate::version::{LibraryVersion, version};
pub use crate::error::{Result, Error};

pub use crate::context::{Context, LogLevel, Hotplug, Registration};
pub use crate::device_list::{DeviceList, Devices};
pub use crate::device::Device;
pub use crate::device_handle::DeviceHandle;

pub use crate::fields::{Speed, TransferType, SyncType, UsageType, Direction, RequestType, Recipient, Version, request_type};
pub use crate::device_descriptor::DeviceDescriptor;
pub use crate::config_descriptor::{ConfigDescriptor, Interfaces};
pub use crate::interface_descriptor::{Interface, InterfaceDescriptors, InterfaceDescriptor, EndpointDescriptors};
pub use crate::endpoint_descriptor::EndpointDescriptor;
pub use crate::language::{Language, PrimaryLanguage, SubLanguage};


#[cfg(test)]
#[macro_use]
mod test_helpers;

#[macro_use]
mod error;
mod version;

mod context;
mod device_list;
mod device;
mod device_handle;

mod fields;
mod device_descriptor;
mod config_descriptor;
mod interface_descriptor;
mod endpoint_descriptor;
mod language;
