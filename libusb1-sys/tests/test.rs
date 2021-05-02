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

#[test]
fn test_fill_control_setup() {
    let mut buf = [0u8; ffi::constants::LIBUSB_CONTROL_SETUP_SIZE + 1];
    unsafe {
        ffi::libusb_fill_control_setup(
            buf.as_mut_ptr(),
            ffi::constants::LIBUSB_REQUEST_TYPE_VENDOR | ffi::constants::LIBUSB_ENDPOINT_OUT,
            0x04,
            0x4e,
            0,
            (buf.len() - ffi::constants::LIBUSB_CONTROL_SETUP_SIZE) as u16,
        );
    }
    buf[ffi::constants::LIBUSB_CONTROL_SETUP_SIZE] = 0x01;
    let setup: *mut ffi::libusb_control_setup = buf.as_mut_ptr() as *mut _;

    assert_eq!(
        unsafe { (*setup).bmRequestType },
        ffi::constants::LIBUSB_REQUEST_TYPE_VENDOR | ffi::constants::LIBUSB_ENDPOINT_OUT
    );
    assert_eq!(unsafe { (*setup).bRequest }, 0x04);
    assert_eq!(unsafe { u16::from_le((*setup).wValue) }, 0x4e);
    assert_eq!(unsafe { u16::from_le((*setup).wIndex) }, 0);
    assert_eq!(unsafe { u16::from_le((*setup).wLength) }, 1);
}

#[test]
fn test_fill_control_transfer() {
    extern "system" fn callback(_transfer: *mut ffi::libusb_transfer) {}

    let mut buf = [0u8; ffi::constants::LIBUSB_CONTROL_SETUP_SIZE + 1];
    unsafe {
        ffi::libusb_fill_control_setup(
            buf.as_mut_ptr(),
            ffi::constants::LIBUSB_REQUEST_TYPE_VENDOR | ffi::constants::LIBUSB_ENDPOINT_OUT,
            0x04,
            0x4e,
            0,
            (buf.len() - ffi::constants::LIBUSB_CONTROL_SETUP_SIZE) as u16,
        );
    }
    buf[ffi::constants::LIBUSB_CONTROL_SETUP_SIZE] = 0x05;

    let mut transfer = std::mem::MaybeUninit::<ffi::libusb_transfer>::uninit();

    unsafe {
        ffi::libusb_fill_control_transfer(
            transfer.as_mut_ptr(),
            std::ptr::null_mut(),
            buf.as_mut_ptr(),
            callback,
            std::ptr::null_mut(),
            1000,
        );
    }
    let transfer = unsafe { &mut transfer.assume_init() };
    assert_eq!(transfer.endpoint, 0);
    assert_eq!(
        transfer.length as usize,
        ffi::constants::LIBUSB_CONTROL_SETUP_SIZE + 1
    );
    assert_eq!(transfer.timeout, 1000);
    assert_eq!(
        transfer.transfer_type,
        ffi::constants::LIBUSB_TRANSFER_TYPE_CONTROL
    );
    assert_eq!(transfer.buffer, buf.as_mut_ptr());

    let data = unsafe {
        std::slice::from_raw_parts(ffi::libusb_control_transfer_get_data(transfer as *mut _), 1)
    };
    assert_eq!(data[0], 0x05);
}

#[test]
fn test_fill_bulk_transfer() {
    extern "system" fn callback(_transfer: *mut ffi::libusb_transfer) {}

    let mut transfer = std::mem::MaybeUninit::<ffi::libusb_transfer>::uninit();
    let mut buf = [5u8; 64];
    unsafe {
        ffi::libusb_fill_bulk_transfer(
            transfer.as_mut_ptr(),
            std::ptr::null_mut(),
            0x80,
            buf.as_mut_ptr(),
            buf.len() as libc::c_int,
            callback,
            std::ptr::null_mut(),
            1000,
        );
    }
    let transfer = unsafe { &transfer.assume_init() };
    assert_eq!(transfer.endpoint, 0x80);
    assert_eq!(transfer.timeout, 1000);
    assert_eq!(
        transfer.transfer_type,
        ffi::constants::LIBUSB_TRANSFER_TYPE_BULK
    );
    assert_eq!(transfer.buffer, buf.as_mut_ptr());
    assert_eq!(transfer.length, buf.len() as libc::c_int);
}
