#![allow(non_camel_case_types)]

extern crate libc;

use libc::{c_void,c_int,c_uint,c_char,c_uchar,ssize_t,timeval};


#[repr(C)]
pub struct libusb_context {
    __private: c_void
}

#[repr(C)]
pub struct libusb_device {
    __private: c_void
}

#[repr(C)]
pub struct libusb_device_handle {
    __private: c_void
}

#[repr(C)]
pub struct libusb_version {
  pub major: u16,
  pub minor: u16,
  pub micro: u16,
  pub nano: u16,
  pub rc: *const c_char,
  pub describe: *const c_char
}

#[allow(non_snake_case)]
#[repr(C)]
pub struct libusb_device_descriptor {
  pub bLength: u8,
  pub bDescriptorType: u8,
  pub bcdUSB: u16,
  pub bDeviceClass: u8,
  pub bDeviceSubClass: u8,
  pub bDeviceProtocol: u8,
  pub bMaxPacketSize0: u8,
  pub idVendor: u16,
  pub idProduct: u16,
  pub bcdDevice: u16,
  pub iManufacturer: u8,
  pub iProduct: u8,
  pub iSerialNumber: u8,
  pub bNumConfigurations: u8
}

#[allow(non_snake_case)]
#[repr(C)]
pub struct libusb_config_descriptor {
  pub bLength: u8,
  pub bDescriptorType: u8,
  pub wTotalLength: u16,
  pub bNumInterfaces: u8,
  pub bConfigurationValue: u8,
  pub iConfiguration: u8,
  pub bmAttributes: u8,
  pub bMaxPower: u8,
  pub interface: *const libusb_interface,
  pub extra: *const c_uchar,
  pub extra_length: c_int
}

#[repr(C)]
pub struct libusb_interface {
  pub altsetting: *const libusb_interface_descriptor,
  pub num_altsetting: c_int
}

#[allow(non_snake_case)]
#[repr(C)]
pub struct libusb_interface_descriptor {
  pub bLength: u8,
  pub bDescriptorType: u8,
  pub bInterfaceNumber: u8,
  pub bAlternateSetting: u8,
  pub bNumEndpoints: u8,
  pub bInterfaceClass: u8,
  pub bInterfaceSubClass: u8,
  pub bInterfaceProtocol: u8,
  pub iInterface: u8,
  pub endpoint: *const libusb_endpoint_descriptor,
  pub extra: *const c_char,
  pub extra_length: c_int
}

#[allow(non_snake_case)]
#[repr(C)]
pub struct libusb_endpoint_descriptor {
  pub bLength: u8,
  pub bDescriptorType: u8,
  pub bEndpointAddress: u8,
  pub bmAttributes: u8,
  pub wMaxPacketSize: u16,
  pub bInterval: u8,
  pub bRefresh: u8,
  pub bSynchAddress: u8,
  pub extra: *const c_char,
  pub extra_length: c_int
}

#[repr(C)]
pub struct libusb_iso_packet_descriptor {
  length: c_uint,
  actual_length: c_uint,
  status: c_int,
}

pub type libusb_transfer_cb_fn = extern "C" fn(*mut libusb_transfer);

#[repr(C)]
pub struct libusb_transfer {
  pub dev_handle: *mut libusb_device_handle,
  pub flags: u8,
  pub endpoint: c_uchar,
  pub transfer_type: c_uchar,
  pub timeout: c_uint,
  pub status: c_int,
  pub length: c_int,
  pub actual_length: c_int,
  pub callback: libusb_transfer_cb_fn,
  pub user_data: *mut c_void,
  pub buffer: *mut c_uchar,
  pub num_iso_packets: c_int,
  pub iso_packet_desc: [libusb_iso_packet_descriptor; 0],
}

// libusb_error
pub const LIBUSB_SUCCESS:             c_int = 0;
pub const LIBUSB_ERROR_IO:            c_int = -1;
pub const LIBUSB_ERROR_INVALID_PARAM: c_int = -2;
pub const LIBUSB_ERROR_ACCESS:        c_int = -3;
pub const LIBUSB_ERROR_NO_DEVICE:     c_int = -4;
pub const LIBUSB_ERROR_NOT_FOUND:     c_int = -5;
pub const LIBUSB_ERROR_BUSY:          c_int = -6;
pub const LIBUSB_ERROR_TIMEOUT:       c_int = -7;
pub const LIBUSB_ERROR_OVERFLOW:      c_int = -8;
pub const LIBUSB_ERROR_PIPE:          c_int = -9;
pub const LIBUSB_ERROR_INTERRUPTED:   c_int = -10;
pub const LIBUSB_ERROR_NO_MEM:        c_int = -11;
pub const LIBUSB_ERROR_NOT_SUPPORTED: c_int = -12;
pub const LIBUSB_ERROR_OTHER:         c_int = -99;

// libusb_transfer_status
pub const LIBUSB_TRANSFER_COMPLETED:  c_int = 0;
pub const LIBUSB_TRANSFER_ERROR:      c_int = 1;
pub const LIBUSB_TRANSFER_TIMED_OUT:  c_int = 2;
pub const LIBUSB_TRANSFER_CANCELLED:  c_int = 3;
pub const LIBUSB_TRANSFER_STALL:      c_int = 4;
pub const LIBUSB_TRANSFER_NO_DEVICE:  c_int = 5;
pub const LIBUSB_TRANSFER_OVERFLOW:   c_int = 6;

pub const LIBUSB_TRANSFER_SHORT_NOT_OK:    u8 = 1<<0;
pub const LIBUSB_TRANSFER_FREE_BUFFER :    u8 = 1<<1;
pub const LIBUSB_TRANSFER_FREE_TRANSFER :  u8 = 1<<2;
pub const LIBUSB_TRANSFER_ADD_ZERO_PACKET: u8 = 1<<3;

// libusb_capability
pub const LIBUSB_CAP_HAS_CAPABILITY:                u32 = 0x0000;
pub const LIBUSB_CAP_HAS_HOTPLUG:                   u32 = 0x0001;
pub const LIBUSB_CAP_HAS_HID_ACCESS:                u32 = 0x0100;
pub const LIBUSB_CAP_SUPPORTS_DETACH_KERNEL_DRIVER: u32 = 0x0101;


//// libusb_log_level
pub const LIBUSB_LOG_LEVEL_NONE:    c_int = 0;
pub const LIBUSB_LOG_LEVEL_ERROR:   c_int = 1;
pub const LIBUSB_LOG_LEVEL_WARNING: c_int = 2;
pub const LIBUSB_LOG_LEVEL_INFO:    c_int = 3;
pub const LIBUSB_LOG_LEVEL_DEBUG:   c_int = 4;


// libusb_class_code
pub const LIBUSB_CLASS_PER_INTERFACE:       u8 = 0;
pub const LIBUSB_CLASS_AUDIO:               u8 = 1;
pub const LIBUSB_CLASS_COMM:                u8 = 2;
pub const LIBUSB_CLASS_HID:                 u8 = 3;
pub const LIBUSB_CLASS_PHYSICAL:            u8 = 5;
pub const LIBUSB_CLASS_PRINTER:             u8 = 7;
pub const LIBUSB_CLASS_IMAGE:               u8 = 6;
pub const LIBUSB_CLASS_MASS_STORAGE:        u8 = 8;
pub const LIBUSB_CLASS_HUB:                 u8 = 9;
pub const LIBUSB_CLASS_DATA:                u8 = 10;
pub const LIBUSB_CLASS_SMART_CARD:          u8 = 0x0B;
pub const LIBUSB_CLASS_CONTENT_SECURITY:    u8 = 0x0D;
pub const LIBUSB_CLASS_VIDEO:               u8 = 0x0E;
pub const LIBUSB_CLASS_PERSONAL_HEALTHCARE: u8 = 0x0F;
pub const LIBUSB_CLASS_DIAGNOSTIC_DEVICE:   u8 = 0xDC;
pub const LIBUSB_CLASS_WIRELESS:            u8 = 0xE0;
pub const LIBUSB_CLASS_APPLICATION:         u8 = 0xFE;
pub const LIBUSB_CLASS_VENDOR_SPEC:         u8 = 0xFF;


// libusb_speed
pub const LIBUSB_SPEED_UNKNOWN: c_int = 0;
pub const LIBUSB_SPEED_LOW:     c_int = 1;
pub const LIBUSB_SPEED_FULL:    c_int = 2;
pub const LIBUSB_SPEED_HIGH:    c_int = 3;
pub const LIBUSB_SPEED_SUPER:   c_int = 4;


// libusb_descriptor_type
pub const LIBUSB_DT_DEVICE:                u8 = 0x01;
pub const LIBUSB_DT_CONFIG:                u8 = 0x02;
pub const LIBUSB_DT_STRING:                u8 = 0x03;
pub const LIBUSB_DT_INTERFACE:             u8 = 0x04;
pub const LIBUSB_DT_ENDPOINT:              u8 = 0x05;
pub const LIBUSB_DT_BOS:                   u8 = 0x0F;
pub const LIBUSB_DT_DEVICE_CAPABILITY:     u8 = 0x10;
pub const LIBUSB_DT_HID:                   u8 = 0x21;
pub const LIBUSB_DT_REPORT:                u8 = 0x22;
pub const LIBUSB_DT_PHYSICAL:              u8 = 0x23;
pub const LIBUSB_DT_HUB:                   u8 = 0x29;
pub const LIBUSB_DT_SUPERSPEED_HUB:        u8 = 0x2A;
pub const LIBUSB_DT_SS_ENDPOINT_COMPANION: u8 = 0x30;


// libusb_endpoint_direction
pub const LIBUSB_ENDPOINT_ADDRESS_MASK: u8 = 0x0F;
pub const LIBUSB_ENDPOINT_DIR_MASK:     u8 = 0x80;
pub const LIBUSB_ENDPOINT_IN:           u8 = 0x80;
pub const LIBUSB_ENDPOINT_OUT:          u8 = 0x00;


// libusb_transfer_type
pub const LIBUSB_TRANSFER_TYPE_MASK:        u8 = 0x03;
pub const LIBUSB_TRANSFER_TYPE_CONTROL:     u8 = 0;
pub const LIBUSB_TRANSFER_TYPE_ISOCHRONOUS: u8 = 1;
pub const LIBUSB_TRANSFER_TYPE_BULK:        u8 = 2;
pub const LIBUSB_TRANSFER_TYPE_INTERRUPT:   u8 = 3;
pub const LIBUSB_TRANSFER_TYPE_BULK_STREAM: u8 = 4;


// libusb_iso_sync_type
pub const LIBUSB_ISO_SYNC_TYPE_MASK:     u8 = 0x0C;
pub const LIBUSB_ISO_SYNC_TYPE_NONE:     u8 = 0;
pub const LIBUSB_ISO_SYNC_TYPE_ASYNC:    u8 = 1;
pub const LIBUSB_ISO_SYNC_TYPE_ADAPTIVE: u8 = 2;
pub const LIBUSB_ISO_SYNC_TYPE_SYNC:     u8 = 3;


// libusb_iso_usage_type
pub const LIBUSB_ISO_USAGE_TYPE_MASK:     u8 = 0x30;
pub const LIBUSB_ISO_USAGE_TYPE_DATA:     u8 = 0;
pub const LIBUSB_ISO_USAGE_TYPE_FEEDBACK: u8 = 1;
pub const LIBUSB_ISO_USAGE_TYPE_IMPLICIT: u8 = 2;


// libusb_request_type
pub const LIBUSB_REQUEST_TYPE_STANDARD: u8 = 0x00 << 5;
pub const LIBUSB_REQUEST_TYPE_CLASS:    u8 = 0x01 << 5;
pub const LIBUSB_REQUEST_TYPE_VENDOR:   u8 = 0x02 << 5;
pub const LIBUSB_REQUEST_TYPE_RESERVED: u8 = 0x03 << 5;


// libusb_request_recipient
pub const LIBUSB_RECIPIENT_DEVICE:    u8 = 0x00;
pub const LIBUSB_RECIPIENT_INTERFACE: u8 = 0x01;
pub const LIBUSB_RECIPIENT_ENDPOINT:  u8 = 0x02;
pub const LIBUSB_RECIPIENT_OTHER:     u8 = 0x03;


// libusb_standard_request
pub const LIBUSB_REQUEST_GET_STATUS:        u8 = 0x00;
pub const LIBUSB_REQUEST_CLEAR_FEATURE:     u8 = 0x01;
pub const LIBUSB_REQUEST_SET_FEATURE:       u8 = 0x03;
pub const LIBUSB_REQUEST_SET_ADDRESS:       u8 = 0x05;
pub const LIBUSB_REQUEST_GET_DESCRIPTOR:    u8 = 0x06;
pub const LIBUSB_REQUEST_SET_DESCRIPTOR:    u8 = 0x07;
pub const LIBUSB_REQUEST_GET_CONFIGURATION: u8 = 0x08;
pub const LIBUSB_REQUEST_SET_CONFIGURATION: u8 = 0x09;
pub const LIBUSB_REQUEST_GET_INTERFACE:     u8 = 0x0A;
pub const LIBUSB_REQUEST_SET_INTERFACE:     u8 = 0x0B;
pub const LIBUSB_REQUEST_SYNCH_FRAME:       u8 = 0x0C;
pub const LIBUSB_REQUEST_SET_SEL:           u8 = 0x30;
pub const LIBUSB_SET_ISOCH_DELAY:           u8 = 0x31;


extern "C" {
  pub fn libusb_get_version() -> *const libusb_version;
  pub fn libusb_has_capability(capability: u32) -> c_int;

  pub fn libusb_init(context: *mut *mut libusb_context) -> c_int;
  pub fn libusb_exit(context: *mut libusb_context);
  pub fn libusb_set_debug(context: *mut libusb_context, level: c_int);

  pub fn libusb_get_device_list(context: *mut libusb_context, list: *mut *const *mut libusb_device) -> ssize_t;
  pub fn libusb_free_device_list(list: *const *mut libusb_device, unref_devices: c_int);
  pub fn libusb_get_parent(dev: *mut libusb_device) -> *mut libusb_device;
  pub fn libusb_get_device(dev_handle: *mut libusb_device_handle) -> *mut libusb_device;

  pub fn libusb_ref_device(dev: *mut libusb_device) -> *mut libusb_device;
  pub fn libusb_unref_device(dev: *mut libusb_device);

  pub fn libusb_get_device_descriptor(dev: *const libusb_device, desc: *mut libusb_device_descriptor) -> c_int;
  pub fn libusb_get_config_descriptor(dev: *const libusb_device, index: u8, config: *mut *const libusb_config_descriptor) -> c_int;
  pub fn libusb_free_config_descriptor(config: *const libusb_config_descriptor);

  pub fn libusb_get_bus_number(dev: *const libusb_device) -> u8;
  pub fn libusb_get_device_address(dev: *const libusb_device) -> u8;
  pub fn libusb_get_device_speed(dev: *const libusb_device) -> c_int;
  pub fn libusb_get_max_packet_size(dev: *const libusb_device, endpoint: c_uchar) -> c_int;
  pub fn libusb_get_max_iso_packet_size(dev: *const libusb_device, endpoint: c_uchar) -> c_int;

  pub fn libusb_open(dev: *const libusb_device, handle: *mut *mut libusb_device_handle) -> c_int;
  pub fn libusb_close(dev_handle: *mut libusb_device_handle);
  pub fn libusb_open_device_with_vid_pid(context: *mut libusb_context, vendor_id: u16, product_id: u16) -> *mut libusb_device_handle;
  pub fn libusb_reset_device(dev_handle: *mut libusb_device_handle) -> c_int;
  pub fn libusb_get_string_descriptor_ascii(dev_handle: *mut libusb_device_handle, desc_index: u8, data: *mut c_uchar, length: c_int) -> c_int;

  pub fn libusb_get_configuration(dev_handle: *mut libusb_device_handle, config: *mut c_int) -> c_int;
  pub fn libusb_set_configuration(dev_handle: *mut libusb_device_handle, config: c_int) -> c_int;

  pub fn libusb_set_auto_detach_kernel_driver(dev_handle: *mut libusb_device_handle, enable: c_int) -> c_int;
  pub fn libusb_kernel_driver_active(dev_handle: *mut libusb_device_handle, interface_number: c_int) -> c_int;
  pub fn libusb_detach_kernel_driver(dev_handle: *mut libusb_device_handle, interface_number: c_int) -> c_int;
  pub fn libusb_attach_kernel_driver(dev_handle: *mut libusb_device_handle, interface_number: c_int) -> c_int;

  pub fn libusb_claim_interface(dev_handle: *mut libusb_device_handle, interface_number: c_int) -> c_int;
  pub fn libusb_release_interface(dev_handle: *mut libusb_device_handle, interface_number: c_int) -> c_int;
  pub fn libusb_set_interface_alt_setting(dev_handle: *mut libusb_device_handle, interface_number: c_int, alternate_setting: c_int) -> c_int;
  pub fn libusb_clear_halt(dev_handle: *mut libusb_device_handle, endpoint: c_uchar) -> c_int;

  pub fn libusb_interrupt_transfer(dev_handle: *mut libusb_device_handle, endpoint: c_uchar, data: *mut c_uchar, length: c_int, transferred: *mut c_int, timeout: c_uint) -> c_int;
  pub fn libusb_bulk_transfer(dev_handle: *mut libusb_device_handle, endpoint: c_uchar, data: *mut c_uchar, length: c_int, transferred: *mut c_int, timeout: c_uint) -> c_int;
  pub fn libusb_control_transfer(dev_handle: *mut libusb_device_handle, request_type: u8, request: u8, value: u16, index: u16, data: *mut c_uchar, length: u16, timeout: c_uint) -> c_int;

  pub fn libusb_alloc_transfer(iso_packets: c_int) -> *mut libusb_transfer;
  pub fn libusb_submit_transfer(transfer: *mut libusb_transfer) -> c_int;
  pub fn libusb_cancel_transfer(transfer: *mut libusb_transfer) -> c_int;
  pub fn libusb_free_transfer(transfer: *mut libusb_transfer);

  pub fn libusb_handle_events(ctx: *mut libusb_context) -> c_int;
  pub fn libusb_handle_events_completed(ctx: *mut libusb_context, completed: *mut c_int) -> c_int;
  pub fn libusb_handle_events_timeout_completed(ctx: *mut libusb_context, tv: *const timeval, completed: *mut c_int) -> c_int;
}


// defined as static inline in libusb.h
pub unsafe fn libusb_get_string_descriptor(dev_handle: *mut libusb_device_handle, desc_index: u8, langid: u16, data: *mut c_uchar, length: c_int) -> c_int
{
  libusb_control_transfer(dev_handle, LIBUSB_ENDPOINT_IN, LIBUSB_REQUEST_GET_DESCRIPTOR, (LIBUSB_DT_STRING as u16) << 8 | desc_index as u16, langid, data, length as u16, 1000)
}
