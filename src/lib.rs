//! This crate provides a safe wrapper around the native `libusb` library.

extern crate bit_set;
extern crate libusb_sys as libusb;
extern crate libc;

pub use ::version::{LibraryVersion,version};
pub use ::error::{Result,Error};

pub use ::context::{Context,LogLevel};
pub use ::device_list::{DeviceList};
pub use ::device_ref::{DeviceRef};
pub use ::device_handle::DeviceHandle;

pub use ::fields::{Version};
pub use ::device::{Device,Speed};
pub use ::configuration::{Configuration};
pub use ::interface::{Interface,InterfaceSetting};
pub use ::endpoint::{Endpoint,TransferType,SyncType,UsageType};
pub use ::request::{Direction,RequestType,Recipient,request_type};
pub use ::language::{Language,PrimaryLanguage,SubLanguage};


#[cfg(test)]
#[macro_use]
mod test_helpers;

#[macro_use]
mod error;
mod version;

mod context;
mod device_list;
mod device_ref;
mod device_handle;

mod fields;
mod device;
mod configuration;
mod interface;
mod endpoint;
mod request;
mod language;
