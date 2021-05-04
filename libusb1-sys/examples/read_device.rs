use libc::{c_int, c_uchar, c_uint};
use libusb1_sys as ffi;

use std::{
    io::{Cursor, Read},
    mem, slice,
    str::FromStr,
};

#[derive(Debug)]
struct Endpoint {
    config: u8,
    iface: u8,
    setting: u8,
    address: u8,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 {
        println!("usage: show_device <vendor-id> <product-id>");
        return;
    }

    let vid: u16 = FromStr::from_str(args[1].as_ref()).unwrap();
    let pid: u16 = FromStr::from_str(args[2].as_ref()).unwrap();

    let mut context = mem::MaybeUninit::<*mut ffi::libusb_context>::uninit();
    let mut device_list = mem::MaybeUninit::<*const *mut ffi::libusb_device>::uninit();

    let context = match unsafe { ffi::libusb_init(context.as_mut_ptr()) } {
        0 => unsafe { context.assume_init() },
        e => panic!("libusb_init: {}", e),
    };

    let handle = unsafe { ffi::libusb_open_device_with_vid_pid(context, vid, pid) };

    if !handle.is_null() {
        match unsafe { ffi::libusb_reset_device(handle) } {
            0 => {
                unsafe { ffi::libusb_set_auto_detach_kernel_driver(handle, 0) };

                let device = unsafe { ffi::libusb_get_device(handle) };
                unsafe { ffi::libusb_ref_device(device) };

                if unsafe { ffi::libusb_get_device_list(context, device_list.as_mut_ptr()) } >= 0 {
                    let device_list = unsafe { device_list.assume_init() };
                    print_device_tree(device);
                    println!();

                    unsafe { ffi::libusb_free_device_list(device_list, 1) };
                }

                let languages = get_language_ids(handle);
                println!("Supported languages: {:?}", languages);

                let mut active_config = mem::MaybeUninit::<c_int>::uninit();
                match unsafe { ffi::libusb_get_configuration(handle, active_config.as_mut_ptr()) } {
                    0 => println!("Active configuration: {}", unsafe {
                        active_config.assume_init()
                    }),
                    e => println!("libusb_get_configuration: {}", e),
                }
                println!();

                match find_readable_endpoint(device, ffi::constants::LIBUSB_TRANSFER_TYPE_INTERRUPT)
                {
                    Some(ep) => read_endpoint(
                        handle,
                        device,
                        ep,
                        ffi::constants::LIBUSB_TRANSFER_TYPE_INTERRUPT,
                    ),
                    None => println!("No readable interrupt endpoint"),
                }
                println!();

                match find_readable_endpoint(device, ffi::constants::LIBUSB_TRANSFER_TYPE_BULK) {
                    Some(ep) => read_endpoint(
                        handle,
                        device,
                        ep,
                        ffi::constants::LIBUSB_TRANSFER_TYPE_BULK,
                    ),
                    None => println!("No readable bulk endpoint"),
                }

                unsafe { ffi::libusb_unref_device(device) };
            }
            e => println!("libusb_reset_device: {}", e),
        }

        unsafe { ffi::libusb_close(handle) };
    }

    unsafe { ffi::libusb_exit(context) };
}

fn print_device_tree(device: *mut ffi::libusb_device) -> usize {
    if device.is_null() {
        return 0;
    }

    let parent = unsafe { ffi::libusb_get_parent(device) };
    let depth = print_device_tree(parent);

    for _ in 0..depth {
        print!("  ");
    }

    let bus = unsafe { ffi::libusb_get_bus_number(device) };
    let address = unsafe { ffi::libusb_get_device_address(device) };

    println!("Bus {:03} Device {:03}", bus, address);

    return depth + 1;
}

fn get_language_ids(handle: *mut ffi::libusb_device_handle) -> Vec<u16> {
    let mut buf = Vec::<u8>::with_capacity(255);
    let len = unsafe {
        ffi::libusb_get_string_descriptor(
            handle,
            0,
            0,
            (&mut buf[..]).as_mut_ptr() as *mut c_uchar,
            buf.capacity() as c_int,
        )
    };

    let mut languages = Vec::<u16>::new();

    if len >= 0 {
        unsafe { buf.set_len(len as usize) };

        if buf.len() >= 2 {
            let num_languages = (buf.len() - 2) / 2;
            languages.reserve(num_languages);

            let mut cursor = Cursor::new(buf);
            cursor.set_position(2);

            for _ in 0..num_languages {
                let mut bytes = Vec::<u8>::with_capacity(2);

                match cursor.read(unsafe {
                    slice::from_raw_parts_mut((&mut bytes[..]).as_mut_ptr(), bytes.capacity())
                }) {
                    Ok(len) => {
                        if len == 2 {
                            unsafe { bytes.set_len(len) };

                            let langid = (bytes[1] as u16) << 8 | (bytes[0] as u16);
                            languages.push(langid)
                        } else {
                            return languages;
                        }
                    }
                    Err(_) => return languages,
                }
            }
        }
    } else {
        println!("libusb_get_string_descriptor: {}", len);
    }

    languages
}

fn find_readable_endpoint(device: *mut ffi::libusb_device, transfer_type: u8) -> Option<Endpoint> {
    let mut device_descriptor = mem::MaybeUninit::<ffi::libusb_device_descriptor>::uninit();

    match unsafe { ffi::libusb_get_device_descriptor(device, device_descriptor.as_mut_ptr()) } {
        0 => {
            let device_descriptor = unsafe { device_descriptor.assume_init() };
            for i in 0..device_descriptor.bNumConfigurations {
                let mut config_ptr =
                    mem::MaybeUninit::<*const ffi::libusb_config_descriptor>::uninit();

                match unsafe {
                    ffi::libusb_get_config_descriptor(device, i, config_ptr.as_mut_ptr())
                } {
                    0 => {
                        let config_descriptor = unsafe { &*config_ptr.assume_init() };
                        let interfaces = unsafe {
                            slice::from_raw_parts(
                                config_descriptor.interface,
                                config_descriptor.bNumInterfaces as usize,
                            )
                        };

                        for iface in interfaces {
                            let settings = unsafe {
                                slice::from_raw_parts(
                                    iface.altsetting,
                                    iface.num_altsetting as usize,
                                )
                            };

                            for iface_descriptor in settings {
                                let endpoints = unsafe {
                                    slice::from_raw_parts(
                                        iface_descriptor.endpoint,
                                        iface_descriptor.bNumEndpoints as usize,
                                    )
                                };

                                for endpoint_descriptor in endpoints {
                                    let is_input = endpoint_descriptor.bEndpointAddress
                                        & ffi::constants::LIBUSB_ENDPOINT_DIR_MASK
                                        == ffi::constants::LIBUSB_ENDPOINT_IN;
                                    let matches_type = endpoint_descriptor.bmAttributes
                                        & ffi::constants::LIBUSB_TRANSFER_TYPE_MASK
                                        == transfer_type;

                                    if is_input && matches_type {
                                        return Some(Endpoint {
                                            config: config_descriptor.bConfigurationValue,
                                            iface: iface_descriptor.bInterfaceNumber,
                                            setting: iface_descriptor.bAlternateSetting,
                                            address: endpoint_descriptor.bEndpointAddress,
                                        });
                                    }
                                }
                            }
                        }
                    }
                    e => println!("libusb_get_config_descriptor: {}", e),
                }
            }

            None
        }
        e => {
            println!("libusb_get_device_descriptor: {}", e);
            None
        }
    }
}

fn read_endpoint(
    handle: *mut ffi::libusb_device_handle,
    device: *mut ffi::libusb_device,
    endpoint: Endpoint,
    transfer_type: u8,
) {
    println!("Reading from endpoint: {:?}", endpoint);

    let has_kernel_driver = unsafe {
        if ffi::libusb_kernel_driver_active(handle, endpoint.iface as c_int) == 1 {
            match ffi::libusb_detach_kernel_driver(handle, endpoint.iface as c_int) {
                0 => (),
                e => println!("libusb_detach_kernel_driver: {}", e),
            }

            true
        } else {
            false
        }
    };

    println!(" - kernel driver? {}", has_kernel_driver);

    match unsafe { ffi::libusb_set_configuration(handle, endpoint.config as c_int) } {
        0 => {
            println!(" - max packet size: {}", unsafe {
                ffi::libusb_get_max_packet_size(device, endpoint.address as c_uchar)
            });
            println!(" - max iso packet size: {}", unsafe {
                ffi::libusb_get_max_iso_packet_size(device, endpoint.address as c_uchar)
            });

            match unsafe { ffi::libusb_claim_interface(handle, endpoint.iface as c_int) } {
                0 => {
                    match unsafe {
                        ffi::libusb_set_interface_alt_setting(
                            handle,
                            endpoint.iface as c_int,
                            endpoint.setting as c_int,
                        )
                    } {
                        0 => {
                            let mut vec = Vec::<u8>::with_capacity(256);
                            let timeout: c_uint = 1000;

                            let mut transferred = mem::MaybeUninit::<c_int>::uninit();

                            match transfer_type {
                                ffi::constants::LIBUSB_TRANSFER_TYPE_INTERRUPT => {
                                    match unsafe {
                                        ffi::libusb_interrupt_transfer(
                                            handle,
                                            endpoint.address as c_uchar,
                                            (&vec[..]).as_ptr() as *mut c_uchar,
                                            vec.capacity() as c_int,
                                            transferred.as_mut_ptr(),
                                            timeout,
                                        )
                                    } {
                                        0 => {
                                            unsafe {
                                                vec.set_len(transferred.assume_init() as usize)
                                            };
                                            println!(" - read: {:?}", vec);
                                        }
                                        e => println!("libusb_interrupt_transfer: {}", e),
                                    }
                                }
                                ffi::constants::LIBUSB_TRANSFER_TYPE_BULK => {
                                    match unsafe {
                                        ffi::libusb_bulk_transfer(
                                            handle,
                                            endpoint.address as c_uchar,
                                            (&vec[..]).as_ptr() as *mut c_uchar,
                                            vec.capacity() as c_int,
                                            transferred.as_mut_ptr(),
                                            timeout,
                                        )
                                    } {
                                        0 => {
                                            unsafe {
                                                vec.set_len(transferred.assume_init() as usize)
                                            };
                                            println!(" - read: {:?}", vec);
                                        }
                                        e => println!("libusb_interrupt_transfer: {}", e),
                                    }
                                }
                                tt => println!(" - can't read endpoint with transfer type {}", tt),
                            }
                        }
                        e => println!("libusb_set_interface_alt_setting: {}", e),
                    }

                    match unsafe { ffi::libusb_release_interface(handle, endpoint.iface as c_int) }
                    {
                        0 => (),
                        e => println!("libusb_release_interface: {}", e),
                    }
                }
                e => println!("libusb_claim_interface: {}", e),
            }
        }
        e => println!("libusb_set_configuration: {}", e),
    }

    if has_kernel_driver {
        match unsafe { ffi::libusb_attach_kernel_driver(handle, endpoint.iface as c_int) } {
            0 => (),
            e => println!("libusb_attach_kernel_driver: {}", e),
        }
    }
}
