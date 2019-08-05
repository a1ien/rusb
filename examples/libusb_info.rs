use libusb1_sys as ffi;
use std::{ffi::CStr, str};

fn main() {
    print_version();
    print_capabilities();
}

fn print_version() {
    let version = unsafe { &*ffi::libusb_get_version() };

    let rc = str::from_utf8(unsafe { CStr::from_ptr(version.rc) }.to_bytes()).unwrap_or("");
    let describe =
        str::from_utf8(unsafe { CStr::from_ptr(version.describe) }.to_bytes()).unwrap_or("");

    println!(
        "libusb v{}.{}.{}.{}{} {}",
        version.major, version.minor, version.micro, version.nano, rc, describe
    );
}

fn print_capabilities() {
    let mut context = std::mem::MaybeUninit::<*mut ffi::libusb_context>::uninit();

    // library must be initialized before calling libusb_has_capabililty()
    match unsafe { ffi::libusb_init(context.as_mut_ptr()) } {
        0 => (),
        e => panic!("libusb_init: {}", e),
    };
    let context = unsafe { context.assume_init() };
    unsafe {
        libusb1_sys::libusb_set_debug(context, ffi::constants::LIBUSB_LOG_LEVEL_DEBUG);
        libusb1_sys::libusb_set_debug(context, ffi::constants::LIBUSB_LOG_LEVEL_INFO);
        libusb1_sys::libusb_set_debug(context, ffi::constants::LIBUSB_LOG_LEVEL_WARNING);
        libusb1_sys::libusb_set_debug(context, ffi::constants::LIBUSB_LOG_LEVEL_ERROR);
        libusb1_sys::libusb_set_debug(context, ffi::constants::LIBUSB_LOG_LEVEL_NONE);
    }

    println!("has capability? {}", unsafe {
        ffi::libusb_has_capability(ffi::constants::LIBUSB_CAP_HAS_CAPABILITY) != 0
    });
    println!("has hotplug? {}", unsafe {
        ffi::libusb_has_capability(ffi::constants::LIBUSB_CAP_HAS_HOTPLUG) != 0
    });
    println!("has HID access? {}", unsafe {
        ffi::libusb_has_capability(ffi::constants::LIBUSB_CAP_HAS_HID_ACCESS) != 0
    });
    println!("supports detach kernel driver? {}", unsafe {
        ffi::libusb_has_capability(ffi::constants::LIBUSB_CAP_SUPPORTS_DETACH_KERNEL_DRIVER) != 0
    });

    unsafe { ffi::libusb_exit(context) };
}
