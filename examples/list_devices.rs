use libc::{c_int, c_uchar};

use std::mem;
use std::ptr;
use std::slice;

fn main() {
    let mut context: *mut libusb1_sys::libusb_context = unsafe { mem::uninitialized() };

    match unsafe { libusb1_sys::libusb_init(&mut context) } {
        0 => (),
        e => panic!("libusb_init: {}", get_error(e)),
    };

    list_devices(context);

    unsafe { libusb1_sys::libusb_exit(context) };
}

fn list_devices(context: *mut libusb1_sys::libusb_context) {
    let mut device_list: *const *mut libusb1_sys::libusb_device = unsafe { mem::uninitialized() };

    let len = unsafe { libusb1_sys::libusb_get_device_list(context, &mut device_list) };

    if len < 0 {
        println!("libusb_get_device_list: {}", get_error(len as c_int));
        return;
    }

    let devs = unsafe { slice::from_raw_parts(device_list, len as usize) };

    for dev in devs {
        display_device(dev);
    }

    unsafe { libusb1_sys::libusb_free_device_list(device_list, 1) };
}

fn display_device(dev: &*mut libusb1_sys::libusb_device) {
    let mut descriptor: libusb1_sys::libusb_device_descriptor = unsafe { mem::uninitialized() };
    let mut handle: *mut libusb1_sys::libusb_device_handle = ptr::null_mut();

    let bus = unsafe { libusb1_sys::libusb_get_bus_number(*dev) };
    let address = unsafe { libusb1_sys::libusb_get_device_address(*dev) };
    let speed = unsafe { libusb1_sys::libusb_get_device_speed(*dev) };

    let has_descriptor =
        match unsafe { libusb1_sys::libusb_get_device_descriptor(*dev, &mut descriptor) } {
            0 => true,
            _ => false,
        };

    if unsafe { libusb1_sys::libusb_open(*dev, &mut handle) } < 0 {
        println!("Couldn't open device, some information will be missing");
        handle = ptr::null_mut();
    }

    print!("Bus {:03} Device {:03}", bus, address);

    if has_descriptor {
        print!(
            " ID {:04x}:{:04x}",
            descriptor.idVendor, descriptor.idProduct
        );
    }

    print!(" {}", get_device_speed(speed));

    if has_descriptor {
        if descriptor.iManufacturer > 0 {
            match get_string_descriptor(handle, descriptor.iManufacturer) {
                Some(s) => print!(" {}", s),
                None => (),
            }
        }

        if descriptor.iProduct > 0 {
            match get_string_descriptor(handle, descriptor.iProduct) {
                Some(s) => print!(" {}", s),
                None => (),
            }
        }

        if descriptor.iSerialNumber > 0 {
            match get_string_descriptor(handle, descriptor.iSerialNumber) {
                Some(s) => print!(" Serial No. {}", s),
                None => (),
            }
        }
    }

    println!("");

    if has_descriptor {
        print_device_descriptor(handle, &descriptor);

        for i in 0..descriptor.bNumConfigurations {
            let mut descriptor: *const libusb1_sys::libusb_config_descriptor =
                unsafe { mem::uninitialized() };

            match unsafe { libusb1_sys::libusb_get_config_descriptor(*dev, i, &mut descriptor) } {
                0 => {
                    let config = unsafe { &*descriptor };
                    let interfaces = unsafe {
                        slice::from_raw_parts(config.interface, config.bNumInterfaces as usize)
                    };

                    print_config_descriptor(handle, config);

                    for iface in interfaces {
                        let iface_descriptors = unsafe {
                            slice::from_raw_parts(iface.altsetting, iface.num_altsetting as usize)
                        };

                        for iface_desc in iface_descriptors {
                            print_interface_descriptor(handle, iface_desc);

                            let endpoints = unsafe {
                                slice::from_raw_parts(
                                    iface_desc.endpoint,
                                    iface_desc.bNumEndpoints as usize,
                                )
                            };

                            for endpoint in endpoints {
                                print_endpoint_descriptor(endpoint);
                            }
                        }
                    }

                    unsafe { libusb1_sys::libusb_free_config_descriptor(descriptor) };
                }
                _ => (),
            }
        }
    }

    if !handle.is_null() {
        unsafe { libusb1_sys::libusb_close(handle) };
    }
}

fn print_device_descriptor(
    handle: *mut libusb1_sys::libusb_device_handle,
    descriptor: &libusb1_sys::libusb_device_descriptor,
) {
    println!("Device Descriptor:");
    println!("  bLength: {:16}", descriptor.bLength);
    println!(
        "  bDescriptorType: {:8} {}",
        descriptor.bDescriptorType,
        get_descriptor_type(descriptor.bDescriptorType)
    );
    println!(
        "  bcdUSB:            {:#06x} {}",
        descriptor.bcdUSB,
        get_bcd_version(descriptor.bcdUSB)
    );
    println!(
        "  bDeviceClass:        {:#04x} {}",
        descriptor.bDeviceClass,
        get_class_type(descriptor.bDeviceClass)
    );
    println!("  bDeviceSubClass: {:8}", descriptor.bDeviceSubClass);
    println!("  bDeviceProtocol: {:8}", descriptor.bDeviceProtocol);
    println!("  bMaxPacketSize0: {:8}", descriptor.bMaxPacketSize0);
    println!("  idVendor:          {:#06x}", descriptor.idVendor);
    println!("  idProduct:         {:#06x}", descriptor.idProduct);
    println!("  bcdDevice:         {:#06x}", descriptor.bcdDevice);
    println!(
        "  iManufacturer: {:10} {}",
        descriptor.iManufacturer,
        get_string_descriptor(handle, descriptor.iManufacturer).unwrap_or(String::new())
    );
    println!(
        "  iProduct: {:15} {}",
        descriptor.iProduct,
        get_string_descriptor(handle, descriptor.iProduct).unwrap_or(String::new())
    );
    println!(
        "  iSerialNumber: {:10} {}",
        descriptor.iSerialNumber,
        get_string_descriptor(handle, descriptor.iSerialNumber).unwrap_or(String::new())
    );
    println!("  bNumConfigurations: {:5}", descriptor.bNumConfigurations);
}

fn print_config_descriptor(
    handle: *mut libusb1_sys::libusb_device_handle,
    descriptor: &libusb1_sys::libusb_config_descriptor,
) {
    println!("  Configuration Descriptor:");
    println!("    bLength: {:16}", descriptor.bLength);
    println!(
        "    bDescriptorType: {:8} {}",
        descriptor.bDescriptorType,
        get_descriptor_type(descriptor.bDescriptorType)
    );
    println!("    wTotalLength: {:11}", descriptor.wTotalLength);
    println!("    bNumInterfaces: {:9}", descriptor.bNumInterfaces);
    println!(
        "    bConfigurationValue: {:4}",
        descriptor.bConfigurationValue
    );
    println!(
        "    iConfiguration: {:9} {}",
        descriptor.iConfiguration,
        get_string_descriptor(handle, descriptor.iConfiguration).unwrap_or(String::new())
    );
    println!("    bmAttributes:        {:#04x}", descriptor.bmAttributes);
    println!(
        "    bMaxPower: {:14} {}",
        descriptor.bMaxPower,
        get_max_power(descriptor.bMaxPower)
    );

    if descriptor.extra_length > 0 {
        let extra =
            unsafe { slice::from_raw_parts(descriptor.extra, descriptor.extra_length as usize) };
        println!("    (extra: {:?})", extra);
    }
}

fn print_interface_descriptor(
    handle: *mut libusb1_sys::libusb_device_handle,
    descriptor: &libusb1_sys::libusb_interface_descriptor,
) {
    println!("    Interface Descriptor:");
    println!("      bLength: {:16}", descriptor.bLength);
    println!(
        "      bDescriptorType: {:8} {}",
        descriptor.bDescriptorType,
        get_descriptor_type(descriptor.bDescriptorType)
    );
    println!("      bInterfaceNumber: {:7}", descriptor.bInterfaceNumber);
    println!(
        "      bAlternateSetting: {:6}",
        descriptor.bAlternateSetting
    );
    println!("      bNumEndpoints: {:10}", descriptor.bNumEndpoints);
    println!(
        "      bInterfaceClass:     {:#04x} {}",
        descriptor.bInterfaceClass,
        get_class_type(descriptor.bInterfaceClass)
    );
    println!(
        "      bInterfaceSubClass: {:5}",
        descriptor.bInterfaceSubClass
    );
    println!(
        "      bInterfaceProtocol: {:5}",
        descriptor.bInterfaceProtocol
    );
    println!(
        "      iInterface: {:13} {}",
        descriptor.iInterface,
        get_string_descriptor(handle, descriptor.iInterface).unwrap_or(String::new())
    );

    if descriptor.extra_length > 0 {
        let extra =
            unsafe { slice::from_raw_parts(descriptor.extra, descriptor.extra_length as usize) };
        println!("    (extra: {:?})", extra);
    }
}

fn print_endpoint_descriptor(descriptor: &libusb1_sys::libusb_endpoint_descriptor) {
    println!("      Endpoint Descriptor:");
    println!("        bLength: {:16}", descriptor.bLength);
    println!(
        "        bDescriptorType: {:8} {}",
        descriptor.bDescriptorType,
        get_descriptor_type(descriptor.bDescriptorType)
    );
    println!(
        "        bEndpointAddress:    {:#04x} {}",
        descriptor.bEndpointAddress,
        get_endpoint(descriptor.bEndpointAddress)
    );
    println!(
        "        bmAttributes:        {:#04x}",
        descriptor.bmAttributes
    );
    println!(
        "          Transfer Type:           {}",
        get_transfer_type(descriptor.bmAttributes)
    );
    println!(
        "          Synch Type:              {}",
        get_synch_type(descriptor.bmAttributes)
    );
    println!(
        "          Usage Type:              {}",
        get_usage_type(descriptor.bmAttributes)
    );
    println!(
        "        wMaxPacketSize:    {:#06x}",
        descriptor.wMaxPacketSize
    );
    println!("        bInterval: {:14}", descriptor.bInterval);
    println!("        bRefresh: {:15}", descriptor.bRefresh);
    println!("        bSynchAddress: {:10}", descriptor.bSynchAddress);

    if descriptor.extra_length > 0 {
        let extra =
            unsafe { slice::from_raw_parts(descriptor.extra, descriptor.extra_length as usize) };
        println!("    (extra: {:?})", extra);
    }
}

fn get_error(err: c_int) -> &'static str {
    match err {
        libusb1_sys::constants::LIBUSB_SUCCESS => "success",
        libusb1_sys::constants::LIBUSB_ERROR_IO => "I/O error",
        libusb1_sys::constants::LIBUSB_ERROR_INVALID_PARAM => "invalid parameter",
        libusb1_sys::constants::LIBUSB_ERROR_ACCESS => "access denied",
        libusb1_sys::constants::LIBUSB_ERROR_NO_DEVICE => "no such device",
        libusb1_sys::constants::LIBUSB_ERROR_NOT_FOUND => "entity not found",
        libusb1_sys::constants::LIBUSB_ERROR_BUSY => "resource busy",
        libusb1_sys::constants::LIBUSB_ERROR_TIMEOUT => "opteration timed out",
        libusb1_sys::constants::LIBUSB_ERROR_OVERFLOW => "overflow error",
        libusb1_sys::constants::LIBUSB_ERROR_PIPE => "pipe error",
        libusb1_sys::constants::LIBUSB_ERROR_INTERRUPTED => "system call interrupted",
        libusb1_sys::constants::LIBUSB_ERROR_NO_MEM => "insufficient memory",
        libusb1_sys::constants::LIBUSB_ERROR_NOT_SUPPORTED => "operation not supported",
        libusb1_sys::constants::LIBUSB_ERROR_OTHER | _ => "other error",
    }
}

fn get_device_speed(speed: c_int) -> &'static str {
    match speed {
        libusb1_sys::constants::LIBUSB_SPEED_SUPER => "5000 Mbps",
        libusb1_sys::constants::LIBUSB_SPEED_HIGH => " 480 Mbps",
        libusb1_sys::constants::LIBUSB_SPEED_FULL => "  12 Mbps",
        libusb1_sys::constants::LIBUSB_SPEED_LOW => " 1.5 Mbps",
        libusb1_sys::constants::LIBUSB_SPEED_UNKNOWN | _ => "(unknown)",
    }
}

fn get_max_power(power: u8) -> String {
    if power > 0 {
        format!("{}mW", power as usize * 2)
    } else {
        String::new()
    }
}

fn get_descriptor_type(desc_type: u8) -> &'static str {
    match desc_type {
        libusb1_sys::constants::LIBUSB_DT_DEVICE => "Device",
        libusb1_sys::constants::LIBUSB_DT_CONFIG => "Configuration",
        libusb1_sys::constants::LIBUSB_DT_STRING => "String",
        libusb1_sys::constants::LIBUSB_DT_INTERFACE => "Interface",
        libusb1_sys::constants::LIBUSB_DT_ENDPOINT => "Endpoint",
        libusb1_sys::constants::LIBUSB_DT_BOS => "BOS",
        libusb1_sys::constants::LIBUSB_DT_DEVICE_CAPABILITY => "Device Capability",
        libusb1_sys::constants::LIBUSB_DT_HID => "HID",
        libusb1_sys::constants::LIBUSB_DT_REPORT => "Report",
        libusb1_sys::constants::LIBUSB_DT_PHYSICAL => "Physical",
        libusb1_sys::constants::LIBUSB_DT_HUB => "HUB",
        libusb1_sys::constants::LIBUSB_DT_SUPERSPEED_HUB => "Superspeed Hub",
        libusb1_sys::constants::LIBUSB_DT_SS_ENDPOINT_COMPANION => "Superspeed Endpoint Companion",
        _ => "",
    }
}

fn get_bcd_version(bcd_version: u16) -> String {
    let digit1 = (bcd_version & 0xF000) >> 12;
    let digit2 = (bcd_version & 0x0F00) >> 8;
    let digit3 = (bcd_version & 0x00F0) >> 4;
    let digit4 = (bcd_version & 0x000F) >> 0;

    if digit1 > 0 {
        format!("{}{}.{}{}", digit1, digit2, digit3, digit4)
    } else {
        format!("{}.{}{}", digit2, digit3, digit4)
    }
}

fn get_class_type(class: u8) -> &'static str {
    match class {
        libusb1_sys::constants::LIBUSB_CLASS_PER_INTERFACE => "(Defined at Interface level)",
        libusb1_sys::constants::LIBUSB_CLASS_AUDIO => "Audio",
        libusb1_sys::constants::LIBUSB_CLASS_COMM => "Comm",
        libusb1_sys::constants::LIBUSB_CLASS_HID => "HID",
        libusb1_sys::constants::LIBUSB_CLASS_PHYSICAL => "Physical",
        libusb1_sys::constants::LIBUSB_CLASS_PRINTER => "Printer",
        libusb1_sys::constants::LIBUSB_CLASS_IMAGE => "Image",
        libusb1_sys::constants::LIBUSB_CLASS_MASS_STORAGE => "Mass Storage",
        libusb1_sys::constants::LIBUSB_CLASS_HUB => "Hub",
        libusb1_sys::constants::LIBUSB_CLASS_DATA => "Data",
        libusb1_sys::constants::LIBUSB_CLASS_SMART_CARD => "Smart Card",
        libusb1_sys::constants::LIBUSB_CLASS_CONTENT_SECURITY => "Content Security",
        libusb1_sys::constants::LIBUSB_CLASS_VIDEO => "Video",
        libusb1_sys::constants::LIBUSB_CLASS_PERSONAL_HEALTHCARE => "Personal Healthcare",
        libusb1_sys::constants::LIBUSB_CLASS_DIAGNOSTIC_DEVICE => "Diagnostic Device",
        libusb1_sys::constants::LIBUSB_CLASS_WIRELESS => "Wireless",
        libusb1_sys::constants::LIBUSB_CLASS_APPLICATION => "Application",
        libusb1_sys::constants::LIBUSB_CLASS_VENDOR_SPEC => "Vendor Specific",
        _ => "",
    }
}

fn get_endpoint(address: u8) -> String {
    let number = address & libusb1_sys::constants::LIBUSB_ENDPOINT_ADDRESS_MASK;

    match address & libusb1_sys::constants::LIBUSB_ENDPOINT_DIR_MASK {
        libusb1_sys::constants::LIBUSB_ENDPOINT_IN => format!("EP {} IN", number),
        libusb1_sys::constants::LIBUSB_ENDPOINT_OUT | _ => format!("EP {} OUT", number),
    }
}

fn get_transfer_type(attributes: u8) -> &'static str {
    match attributes & libusb1_sys::constants::LIBUSB_TRANSFER_TYPE_MASK {
        libusb1_sys::constants::LIBUSB_TRANSFER_TYPE_CONTROL => "Control",
        libusb1_sys::constants::LIBUSB_TRANSFER_TYPE_ISOCHRONOUS => "Isochronous",
        libusb1_sys::constants::LIBUSB_TRANSFER_TYPE_BULK => "Bulk",
        libusb1_sys::constants::LIBUSB_TRANSFER_TYPE_INTERRUPT => "Interrupt",
        libusb1_sys::constants::LIBUSB_TRANSFER_TYPE_BULK_STREAM => "Bulk Stream",
        _ => "",
    }
}

fn get_synch_type(attributes: u8) -> &'static str {
    match (attributes & libusb1_sys::constants::LIBUSB_ISO_SYNC_TYPE_MASK) >> 2 {
        libusb1_sys::constants::LIBUSB_ISO_SYNC_TYPE_NONE => "None",
        libusb1_sys::constants::LIBUSB_ISO_SYNC_TYPE_ASYNC => "Async",
        libusb1_sys::constants::LIBUSB_ISO_SYNC_TYPE_ADAPTIVE => "Adaptive",
        libusb1_sys::constants::LIBUSB_ISO_SYNC_TYPE_SYNC => "Sync",
        _ => "",
    }
}

fn get_usage_type(attributes: u8) -> &'static str {
    match (attributes & libusb1_sys::constants::LIBUSB_ISO_USAGE_TYPE_MASK) >> 4 {
        libusb1_sys::constants::LIBUSB_ISO_USAGE_TYPE_DATA => "Data",
        libusb1_sys::constants::LIBUSB_ISO_USAGE_TYPE_FEEDBACK => "Feedback",
        libusb1_sys::constants::LIBUSB_ISO_USAGE_TYPE_IMPLICIT => "Implicit",
        _ => "",
    }
}

fn get_string_descriptor(
    handle: *mut libusb1_sys::libusb_device_handle,
    desc_index: u8,
) -> Option<String> {
    if handle.is_null() || desc_index == 0 {
        return None;
    }

    let mut vec = Vec::<u8>::with_capacity(256);
    let ptr = (&mut vec[..]).as_mut_ptr();

    let len = unsafe {
        libusb1_sys::libusb_get_string_descriptor_ascii(
            handle,
            desc_index,
            ptr as *mut c_uchar,
            vec.capacity() as c_int,
        )
    };

    if len > 0 {
        unsafe { vec.set_len(len as usize) };

        match String::from_utf8(vec) {
            Ok(s) => Some(s),
            Err(_) => None,
        }
    } else {
        None
    }
}
