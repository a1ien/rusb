use std::{ffi::CStr, mem, str};

fn main() {
    print_version();
    print_capabilities();
}

fn print_version() {
    let version: &libusb1_sys::libusb_version =
        unsafe { mem::transmute(libusb1_sys::libusb_get_version()) };

    let rc = str::from_utf8(unsafe { CStr::from_ptr(version.rc) }.to_bytes()).unwrap_or("");
    let describe =
        str::from_utf8(unsafe { CStr::from_ptr(version.describe) }.to_bytes()).unwrap_or("");

    println!(
        "libusb v{}.{}.{}.{}{} {}",
        version.major, version.minor, version.micro, version.nano, rc, describe
    );
}

fn print_capabilities() {
    let mut context: *mut libusb1_sys::libusb_context = unsafe { mem::uninitialized() };

    // library must be initialized before calling libusb_has_capabililty()
    match unsafe { libusb1_sys::libusb_init(&mut context) } {
        0 => (),
        e => panic!("libusb_init: {}", e),
    };

    unsafe {
        libusb1_sys::libusb_set_debug(context, libusb1_sys::constants::LIBUSB_LOG_LEVEL_DEBUG);
        libusb1_sys::libusb_set_debug(context, libusb1_sys::constants::LIBUSB_LOG_LEVEL_INFO);
        libusb1_sys::libusb_set_debug(context, libusb1_sys::constants::LIBUSB_LOG_LEVEL_WARNING);
        libusb1_sys::libusb_set_debug(context, libusb1_sys::constants::LIBUSB_LOG_LEVEL_ERROR);
        libusb1_sys::libusb_set_debug(context, libusb1_sys::constants::LIBUSB_LOG_LEVEL_NONE);
    }

    println!("has capability? {}", unsafe {
        libusb1_sys::libusb_has_capability(libusb1_sys::constants::LIBUSB_CAP_HAS_CAPABILITY)
    });
    println!("has hotplug? {}", unsafe {
        libusb1_sys::libusb_has_capability(libusb1_sys::constants::LIBUSB_CAP_HAS_HOTPLUG)
    });
    println!("has HID access? {}", unsafe {
        libusb1_sys::libusb_has_capability(libusb1_sys::constants::LIBUSB_CAP_HAS_HID_ACCESS)
    });
    println!("supports detach kernel driver? {}", unsafe {
        libusb1_sys::libusb_has_capability(
            libusb1_sys::constants::LIBUSB_CAP_SUPPORTS_DETACH_KERNEL_DRIVER,
        )
    });

    unsafe { libusb1_sys::libusb_exit(context) };
}
