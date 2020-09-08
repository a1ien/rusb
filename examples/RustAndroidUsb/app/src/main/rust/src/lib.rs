#![cfg(target_os = "android")]
#![allow(non_snake_case)]

extern crate android_logger;
#[macro_use]
extern crate log;

use android_logger::Config;
use jni::JNIEnv;
use jni::objects::JObject;
use jni::sys::{jintArray, jstring};
use log::Level;
use rusb::{DeviceList, GlobalContext, DeviceHandle, UsbContext};
use libusb1_sys::{libusb_context, libusb_device_handle, libusb_wrap_sys_device};
use std::ptr::null_mut;

#[no_mangle]
pub unsafe extern fn Java_com_github_a1ien_rusb_example_rustandroidusb_MainActivity_init(
    env: JNIEnv,
    _obj: JObject,
) {
    android_logger::init_once(Config::default().with_min_level(Level::Debug));
}

#[no_mangle]
pub unsafe extern fn Java_com_github_a1ien_rusb_example_rustandroidusb_MainActivity_listNative(
    env: JNIEnv,
    _obj: JObject,
) -> jstring {
    android_logger::init_once(Config::default().with_min_level(Level::Debug));

    let mut str = "".to_owned();
    str.push_str(format!("Listing Native devices:\n").as_str());
    for device in DeviceList::new().unwrap().iter() {
        str.push_str(format!("Bus {:03} Device {:03}", device.bus_number(), device.address()).as_str());
    }
    **env.new_string(str).unwrap()
}

#[no_mangle]
pub unsafe extern fn Java_com_github_a1ien_rusb_example_rustandroidusb_MainActivity_listAndroid(
    env: JNIEnv,
    _obj: JObject,
    fds: jintArray,
) -> jstring {
    android_logger::init_once(Config::default().with_min_level(Level::Debug));

    let mut str = "".to_owned();
    let len = env.get_array_length(fds).unwrap();
    let mut ints = vec![0i32; len as usize];
    env.get_int_array_region(fds, 0, &mut ints);
    str.push_str(format!("Listing Android devices:\n").as_str());
    for fd in ints {
        let ctx: *mut libusb_context = GlobalContext::default().as_raw();
        let fd = fd as isize as *mut i32;
        let mut handle: *mut libusb_device_handle = null_mut();
        let res = libusb_wrap_sys_device(ctx, fd, &mut handle);
        if res != 0 {
            error!("libusb error code: {}", res);
            continue;
        }
        let ptr = std::ptr::NonNull::new(handle).unwrap();
        let mut handle = DeviceHandle::from_libusb(GlobalContext::default(), ptr);
        let device = handle.device();
        let device_desc = match device.device_descriptor() {
            Ok(d) => d,
            Err(_) => continue,
        };
        str.push_str(format!(
            "Bus {:03} Device {:03} ID {:04x}:{:04x}\n",
            device.bus_number(),
            device.address(),
            device_desc.vendor_id(),
            device_desc.product_id()
        ).as_str());
    }
    **env.new_string(str).unwrap()
}
