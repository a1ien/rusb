//! This crate provides a safe wrapper around the native `libusb` library.

extern crate bit_set;
extern crate libusb_sys as libusb;
extern crate libc;

pub use ::version::{LibraryVersion,version};
pub use ::error::{Result,Error};

pub use ::context::{Context,LogLevel};
pub use ::device_list::{DeviceList};
pub use ::device::{Device};
pub use ::device_handle::DeviceHandle;

pub use ::fields::{Speed,TransferType,SyncType,UsageType,Direction,RequestType,Recipient,Version,request_type};
pub use ::device_descriptor::{DeviceDescriptor};
pub use ::config_descriptor::{ConfigDescriptor,Interfaces};
pub use ::interface_descriptor::{Interface,InterfaceDescriptors,InterfaceDescriptor,EndpointDescriptors};
pub use ::endpoint_descriptor::{EndpointDescriptor};
pub use ::language::{Language,PrimaryLanguage,SubLanguage};


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
