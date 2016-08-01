pub use std::ptr;

macro_rules! merge {
    ($default:expr => $($field:ident : $value:expr),*) => {
        {
            let mut x = $default;
            $( x.$field = $value; )*

            x
        }
    }
}

#[macro_export]
macro_rules! endpoint_descriptor {
    ($($key:ident : $value:expr),*) => {
        merge!(
            ::libusb::libusb_endpoint_descriptor {
                bLength:          7,
                bDescriptorType:  0x05,
                bEndpointAddress: 0x00,
                bmAttributes:     0x00,
                wMaxPacketSize:   16,
                bInterval:        1,
                bRefresh:         1,
                bSynchAddress:    0,
                extra:            $crate::test_helpers::ptr::null(),
                extra_length:     0
            } => $($key: $value),*
        )
    }
}

#[macro_export]
macro_rules! interface_descriptor {
    ($($key:ident : $value:expr),*) => {
        merge!(
            ::libusb::libusb_interface_descriptor {
                bLength:            9,
                bDescriptorType:    0x04,
                bInterfaceNumber:   0,
                bAlternateSetting:  0,
                bNumEndpoints:      0,
                bInterfaceClass:    0,
                bInterfaceSubClass: 0,
                bInterfaceProtocol: 0,
                iInterface:         0,
                endpoint:           $crate::test_helpers::ptr::null(),
                extra:              $crate::test_helpers::ptr::null(),
                extra_length:       0
            } => $($key: $value),*
        )
    };
    ($($endpoint:expr),+) => {
        {
            let endpoints = vec![$($endpoint),+];

            let r = ::libusb::libusb_interface_descriptor {
                bLength:            9,
                bDescriptorType:    0x04,
                bInterfaceNumber:   0,
                bAlternateSetting:  0,
                bNumEndpoints:      endpoints.len() as u8,
                bInterfaceClass:    0,
                bInterfaceSubClass: 0,
                bInterfaceProtocol: 0,
                iInterface:         0,
                endpoint:           (&endpoints[..]).as_ptr(),
                extra:              $crate::test_helpers::ptr::null(),
                extra_length:       0
            };

            // leak the Vec so the returned pointer remains valid
            ::std::mem::forget(endpoints);
            r
        }
    }
}

#[macro_export]
macro_rules! interface {
    ($($descriptor:expr),*) => {
        {
            let descriptors = vec![$($descriptor),*];

            let r = ::libusb::libusb_interface {
                altsetting:     descriptors.as_ptr(),
                num_altsetting: descriptors.len() as ::libc::c_int
            };

            // leak the Vec so the returned pointer remains valid
            ::std::mem::forget(descriptors);
            r
        }
    }
}

#[macro_export]
macro_rules! config_descriptor {
    ($($key:ident : $value:expr),*) => {
        merge!(
            ::libusb::libusb_config_descriptor {
                bLength:             9,
                bDescriptorType:     0x02,
                wTotalLength:        9,
                bNumInterfaces:      0,
                bConfigurationValue: 0,
                iConfiguration:      0,
                bmAttributes:        0x00,
                bMaxPower:           10,
                interface:           $crate::test_helpers::ptr::null(),
                extra:               $crate::test_helpers::ptr::null(),
                extra_length:        0
            } => $($key: $value),*
        )
    };
    ($($interface:expr),+) => {
        {
            let interfaces = vec![$($interface),+];

            let r = ::libusb::libusb_config_descriptor {
                bLength:             9,
                bDescriptorType:     0x02,
                wTotalLength:        9,
                bNumInterfaces:      interfaces.len() as u8,
                bConfigurationValue: 0,
                iConfiguration:      0,
                bmAttributes:        0x00,
                bMaxPower:           10,
                interface:           (&interfaces[..]).as_ptr(),
                extra:               $crate::test_helpers::ptr::null(),
                extra_length:        0
            };

            // leak the Vec so the returned pointer remains valid
            ::std::mem::forget(interfaces);
            r
        }
    }
}

#[macro_export]
macro_rules! device_descriptor {
    ($($key:ident : $value:expr),*) => {
        merge!(
            ::libusb::libusb_device_descriptor {
                bLength:            18,
                bDescriptorType:    0x01,
                bcdUSB:             0x0110,
                bDeviceClass:       0,
                bDeviceSubClass:    0,
                bDeviceProtocol:    0,
                bMaxPacketSize0:    16,
                idVendor:           0x1234,
                idProduct:          0x5678,
                bcdDevice:          0x0123,
                iManufacturer:      0,
                iProduct:           0,
                iSerialNumber:      0,
                bNumConfigurations: 1
            } => $($key: $value),*
        )
    }
}
