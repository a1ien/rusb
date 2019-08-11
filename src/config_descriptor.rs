use std::{fmt, slice};

use libusb1_sys::*;

use crate::interface_descriptor::{self, Interface};

/// Describes a configuration.
pub struct ConfigDescriptor {
    descriptor: *const libusb_config_descriptor,
}

impl Drop for ConfigDescriptor {
    fn drop(&mut self) {
        unsafe {
            libusb_free_config_descriptor(self.descriptor);
        }
    }
}

unsafe impl Sync for ConfigDescriptor {}
unsafe impl Send for ConfigDescriptor {}

impl ConfigDescriptor {
    /// Returns the configuration number.
    pub fn number(&self) -> u8 {
        unsafe { (*self.descriptor).bConfigurationValue }
    }

    /// Returns the device's maximum power consumption (in milliamps) in this configuration.
    pub fn max_power(&self) -> u16 {
        unsafe { u16::from((*self.descriptor).bMaxPower) * 2 }
    }

    /// Indicates if the device is self-powered in this configuration.
    pub fn self_powered(&self) -> bool {
        unsafe { (*self.descriptor).bmAttributes & 0x40 != 0 }
    }

    /// Indicates if the device has remote wakeup capability in this configuration.
    pub fn remote_wakeup(&self) -> bool {
        unsafe { (*self.descriptor).bmAttributes & 0x20 != 0 }
    }

    /// Returns the index of the string descriptor that describes the configuration.
    pub fn description_string_index(&self) -> Option<u8> {
        unsafe {
            match (*self.descriptor).iConfiguration {
                0 => None,
                n => Some(n),
            }
        }
    }

    /// Returns the number of interfaces for this configuration.
    pub fn num_interfaces(&self) -> u8 {
        unsafe { (*self.descriptor).bNumInterfaces }
    }

    /// Returns a collection of the configuration's interfaces.
    pub fn interfaces(&self) -> Interfaces {
        let interfaces = unsafe {
            slice::from_raw_parts(
                (*self.descriptor).interface,
                (*self.descriptor).bNumInterfaces as usize,
            )
        };

        Interfaces {
            iter: interfaces.iter(),
        }
    }

    /// Returns the unknown 'extra' bytes that libusb does not understand.
    pub fn extra(&self) -> Option<&[u8]> {
        unsafe {
            match (*self.descriptor).extra_length {
                len if len > 0 => Some(slice::from_raw_parts(
                    (*self.descriptor).extra,
                    len as usize,
                )),
                _ => None,
            }
        }
    }
}

impl fmt::Debug for ConfigDescriptor {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let mut debug = fmt.debug_struct("ConfigDescriptor");

        let descriptor: &libusb_config_descriptor = unsafe { &*self.descriptor };

        debug.field("bLength", &descriptor.bLength);
        debug.field("bDescriptorType", &descriptor.bDescriptorType);
        debug.field("wTotalLength", &descriptor.wTotalLength);
        debug.field("bNumInterfaces", &descriptor.bNumInterfaces);
        debug.field("bConfigurationValue", &descriptor.bConfigurationValue);
        debug.field("iConfiguration", &descriptor.iConfiguration);
        debug.field("bmAttributes", &descriptor.bmAttributes);
        debug.field("bMaxPower", &descriptor.bMaxPower);
        debug.field("extra", &self.extra());

        debug.finish()
    }
}

/// Iterator over a configuration's interfaces.
pub struct Interfaces<'a> {
    iter: slice::Iter<'a, libusb_interface>,
}

impl<'a> Iterator for Interfaces<'a> {
    type Item = Interface<'a>;

    fn next(&mut self) -> Option<Interface<'a>> {
        self.iter
            .next()
            .map(|interface| unsafe { interface_descriptor::from_libusb(interface) })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

#[doc(hidden)]
pub(crate) unsafe fn from_libusb(config: *const libusb_config_descriptor) -> ConfigDescriptor {
    ConfigDescriptor { descriptor: config }
}

#[cfg(test)]
mod test {
    use std::mem;

    // The Drop trait impl calls libusb_free_config_descriptor(), which would attempt to free
    // unallocated memory for a stack-allocated config descriptor. Allocating a config descriptor
    // is not a simple malloc()/free() inside libusb. Mimicking libusb's allocation would be
    // error-prone, difficult to maintain, and provide little benefit for the tests. It's easier to
    // use mem::forget() to prevent the Drop trait impl from running. The config descriptor passed
    // as `$config` should be stack-allocated to prevent memory leaks in the test suite.
    macro_rules! with_config {
        ($name:ident : $config:expr => $body:block) => {{
            let $name = unsafe { super::from_libusb(&$config) };
            $body;
            mem::forget($name);
        }};
    }

    #[test]
    fn it_has_number() {
        with_config!(config: config_descriptor!(bConfigurationValue: 42) => {
            assert_eq!(42, config.number());
        });
    }

    #[test]
    fn it_has_max_power() {
        with_config!(config: config_descriptor!(bMaxPower: 21) => {
            assert_eq!(42, config.max_power());
        });
    }

    #[test]
    fn it_interprets_self_powered_bit_in_attributes() {
        with_config!(config: config_descriptor!(bmAttributes: 0b0000_0000) => {
            assert_eq!(false, config.self_powered());
        });

        with_config!(config: config_descriptor!(bmAttributes: 0b0100_0000) => {
            assert_eq!(true, config.self_powered());
        });
    }

    #[test]
    fn it_interprets_remote_wakeup_bit_in_attributes() {
        with_config!(config: config_descriptor!(bmAttributes: 0b0000_0000) => {
            assert_eq!(false, config.remote_wakeup());
        });

        with_config!(config: config_descriptor!(bmAttributes: 0b0010_0000) => {
            assert_eq!(true, config.remote_wakeup());
        });
    }

    #[test]
    fn it_has_description_string_index() {
        with_config!(config: config_descriptor!(iConfiguration: 42) => {
            assert_eq!(Some(42), config.description_string_index());
        });
    }

    #[test]
    fn it_handles_missing_description_string_index() {
        with_config!(config: config_descriptor!(iConfiguration: 0) => {
            assert_eq!(None, config.description_string_index());
        });
    }

    #[test]
    fn it_has_num_interfaces() {
        let interface1 = interface!(interface_descriptor!(bInterfaceNumber: 1));
        let interface2 = interface!(interface_descriptor!(bInterfaceNumber: 2));

        with_config!(config: config_descriptor!(interface1, interface2) => {
            assert_eq!(2, config.num_interfaces());
        });
    }

    #[test]
    fn it_has_interfaces() {
        let interface = interface!(interface_descriptor!(bInterfaceNumber: 1));

        with_config!(config: config_descriptor!(interface) => {
            let interface_numbers = config.interfaces().map(|interface| {
                interface.number()
            }).collect::<Vec<_>>();

            assert_eq!(vec![1], interface_numbers);
        });
    }
}
