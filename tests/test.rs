use libusb1_sys as ffi;

#[test]
fn test_version() {
    use std::{ffi::CStr, str};
    let version = unsafe { &*ffi::libusb_get_version() };
    let rc = str::from_utf8(unsafe { CStr::from_ptr(version.rc) }.to_bytes()).unwrap_or("");
    let describe =
        str::from_utf8(unsafe { CStr::from_ptr(version.describe) }.to_bytes()).unwrap_or("");

    assert_eq!(version.major, 1);
    assert_eq!(version.minor, 0);
    println!(
        "libusb v{}.{}.{}.{}{} {}",
        version.major, version.minor, version.micro, version.nano, rc, describe
    );
}

#[test]
fn test_init_and_exit() {
    let mut context: *mut ffi::libusb_context = std::ptr::null_mut();
    for i in 0..=100 {
        match unsafe { ffi::libusb_init(&mut context) } {
            0 => (),
            err => panic!("Failed to init libusb on iteration {}: {}", i, err),
        }
        unsafe {
            ffi::libusb_exit(context);
        }
        context = std::ptr::null_mut();
    }
}

#[test]
fn test_get_device_list() {
    let mut context = std::mem::MaybeUninit::<*mut ffi::libusb_context>::uninit();
    match unsafe { ffi::libusb_init(context.as_mut_ptr()) } {
        0 => (),
        err => panic!("Failed to init libusb {}", err),
    }
    let mut list = std::mem::MaybeUninit::<*const *mut ffi::libusb_device>::uninit();
    let list_size =
        unsafe { ffi::libusb_get_device_list(context.assume_init(), list.as_mut_ptr()) };
    if list_size < 0 || unsafe { list.assume_init().is_null() } {
        panic!("Failed to get device list {} {:p}", -list_size, unsafe {
            list.assume_init()
        });
    }
    unsafe {
        ffi::libusb_free_device_list(list.assume_init(), 1);
    }
    unsafe {
        ffi::libusb_exit(context.assume_init());
    }
}
