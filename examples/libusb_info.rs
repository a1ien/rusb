extern crate libusb_sys as ffi;

use std::mem;
use std::str;
use std::ffi::CStr;

fn main() {
  print_version();
  print_capabilities();
}

fn print_version() {
  let version: &ffi::libusb_version = unsafe {
      mem::transmute(::ffi::libusb_get_version())
  };

  let rc       = str::from_utf8(unsafe { CStr::from_ptr(version.rc)       }.to_bytes()).unwrap_or("");
  let describe = str::from_utf8(unsafe { CStr::from_ptr(version.describe) }.to_bytes()).unwrap_or("");

  println!("libusb v{}.{}.{}.{}{} {}", version.major, version.minor, version.micro, version.nano, rc, describe);
}

fn print_capabilities() {
  let mut context: *mut ::ffi::libusb_context = unsafe { mem::uninitialized() };

  // library must be initialized before calling libusb_has_capabililty()
  match unsafe { ::ffi::libusb_init(&mut context) } {
    0 => (),
    e => panic!("libusb_init: {}", e)
  };

  unsafe {
    ::ffi::libusb_set_debug(context, ::ffi::LIBUSB_LOG_LEVEL_DEBUG);
    ::ffi::libusb_set_debug(context, ::ffi::LIBUSB_LOG_LEVEL_INFO);
    ::ffi::libusb_set_debug(context, ::ffi::LIBUSB_LOG_LEVEL_WARNING);
    ::ffi::libusb_set_debug(context, ::ffi::LIBUSB_LOG_LEVEL_ERROR);
    ::ffi::libusb_set_debug(context, ::ffi::LIBUSB_LOG_LEVEL_NONE);
  }

  println!("has capability? {}", unsafe { ::ffi::libusb_has_capability(::ffi::LIBUSB_CAP_HAS_CAPABILITY) });
  println!("has hotplug? {}", unsafe { ::ffi::libusb_has_capability(::ffi::LIBUSB_CAP_HAS_HOTPLUG) });
  println!("has HID access? {}", unsafe { ::ffi::libusb_has_capability(::ffi::LIBUSB_CAP_HAS_HID_ACCESS) });
  println!("supports detach kernel driver? {}", unsafe { ::ffi::libusb_has_capability(::ffi::LIBUSB_CAP_SUPPORTS_DETACH_KERNEL_DRIVER) });

  unsafe { ::ffi::libusb_exit(context) };
}
