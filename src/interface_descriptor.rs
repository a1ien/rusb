use std::{fmt, slice};

use libusb1_sys::{libusb_endpoint_descriptor, libusb_interface, libusb_interface_descriptor};

use crate::endpoint_descriptor::{self, EndpointDescriptor};

/// A device interface.
///
/// An interface can have several descriptors, each describing an alternate setting of the
/// interface.
pub struct Interface<'a> {
    descriptors: &'a [libusb_interface_descriptor],
}

impl<'a> Interface<'a> {
    /// Returns the interface's number.
    pub fn number(&self) -> u8 {
        self.descriptors[0].bInterfaceNumber
    }

    /// Returns an iterator over the interface's descriptors.
    pub fn descriptors(&self) -> InterfaceDescriptors<'a> {
        InterfaceDescriptors {
            iter: self.descriptors.iter(),
        }
    }
}

/// Iterator over an interface's descriptors.
pub struct InterfaceDescriptors<'a> {
    iter: slice::Iter<'a, libusb_interface_descriptor>,
}

impl<'a> Iterator for InterfaceDescriptors<'a> {
    type Item = InterfaceDescriptor<'a>;

    fn next(&mut self) -> Option<InterfaceDescriptor<'a>> {
        self.iter
            .next()
            .map(|descriptor| InterfaceDescriptor { descriptor })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

/// Describes an alternate setting for an interface.
pub struct InterfaceDescriptor<'a> {
    descriptor: &'a libusb_interface_descriptor,
}

impl<'a> InterfaceDescriptor<'a> {
    /// Returns the interface's number.
    pub fn interface_number(&self) -> u8 {
        self.descriptor.bInterfaceNumber
    }

    /// Returns the alternate setting number.
    pub fn setting_number(&self) -> u8 {
        self.descriptor.bAlternateSetting
    }

    /// Returns the interface's class code.
    pub fn class_code(&self) -> u8 {
        self.descriptor.bInterfaceClass
    }

    /// Returns the interface's sub class code.
    pub fn sub_class_code(&self) -> u8 {
        self.descriptor.bInterfaceSubClass
    }

    /// Returns the interface's protocol code.
    pub fn protocol_code(&self) -> u8 {
        self.descriptor.bInterfaceProtocol
    }

    /// Returns the index of the string descriptor that describes the interface.
    pub fn description_string_index(&self) -> Option<u8> {
        match self.descriptor.iInterface {
            0 => None,
            n => Some(n),
        }
    }

    /// Returns the number of endpoints belonging to this interface.
    pub fn num_endpoints(&self) -> u8 {
        self.descriptor.bNumEndpoints
    }

    /// Returns an iterator over the interface's endpoint descriptors.
    pub fn endpoint_descriptors(&self) -> EndpointDescriptors {
        let endpoints = unsafe {
            slice::from_raw_parts(
                self.descriptor.endpoint,
                self.descriptor.bNumEndpoints as usize,
            )
        };

        EndpointDescriptors {
            iter: endpoints.iter(),
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

impl<'a> fmt::Debug for InterfaceDescriptor<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let mut debug = fmt.debug_struct("InterfaceDescriptor");

        debug.field("bLength", &self.descriptor.bLength);
        debug.field("bDescriptorType", &self.descriptor.bDescriptorType);
        debug.field("bInterfaceNumber", &self.descriptor.bInterfaceNumber);
        debug.field("bAlternateSetting", &self.descriptor.bAlternateSetting);
        debug.field("bNumEndpoints", &self.descriptor.bNumEndpoints);
        debug.field("bInterfaceClass", &self.descriptor.bInterfaceClass);
        debug.field("bInterfaceSubClass", &self.descriptor.bInterfaceSubClass);
        debug.field("bInterfaceProtocol", &self.descriptor.bInterfaceProtocol);
        debug.field("iInterface", &self.descriptor.iInterface);

        debug.finish()
    }
}

/// Iterator over an interface's endpoint descriptors.
pub struct EndpointDescriptors<'a> {
    iter: slice::Iter<'a, libusb_endpoint_descriptor>,
}

impl<'a> Iterator for EndpointDescriptors<'a> {
    type Item = EndpointDescriptor<'a>;

    fn next(&mut self) -> Option<EndpointDescriptor<'a>> {
        self.iter
            .next()
            .map(|endpoint| endpoint_descriptor::from_libusb(endpoint))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

#[doc(hidden)]
pub(crate) unsafe fn from_libusb(interface: &libusb_interface) -> Interface {
    let descriptors =
        slice::from_raw_parts(interface.altsetting, interface.num_altsetting as usize);
    debug_assert!(!descriptors.is_empty());

    Interface { descriptors }
}

#[cfg(test)]
mod test {
    #[test]
    fn it_has_interface_number() {
        assert_eq!(
            42,
            unsafe { super::from_libusb(&interface!(interface_descriptor!(bInterfaceNumber: 42))) }
                .number()
        );
    }

    #[test]
    fn it_has_interface_number_in_descriptor() {
        assert_eq!(
            vec!(42),
            unsafe { super::from_libusb(&interface!(interface_descriptor!(bInterfaceNumber: 42))) }
                .descriptors()
                .map(|setting| setting.interface_number())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn it_has_alternate_setting_number() {
        assert_eq!(
            vec!(42),
            unsafe {
                super::from_libusb(&interface!(interface_descriptor!(bAlternateSetting: 42)))
            }
            .descriptors()
            .map(|setting| setting.setting_number())
            .collect::<Vec<_>>()
        );
    }

    #[test]
    fn it_has_class_code() {
        assert_eq!(
            vec!(42),
            unsafe { super::from_libusb(&interface!(interface_descriptor!(bInterfaceClass: 42))) }
                .descriptors()
                .map(|setting| setting.class_code())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn it_has_sub_class_code() {
        assert_eq!(
            vec!(42),
            unsafe {
                super::from_libusb(&interface!(interface_descriptor!(bInterfaceSubClass: 42)))
            }
            .descriptors()
            .map(|setting| setting.sub_class_code())
            .collect::<Vec<_>>()
        );
    }

    #[test]
    fn it_has_protocol_code() {
        assert_eq!(
            vec!(42),
            unsafe {
                super::from_libusb(&interface!(interface_descriptor!(bInterfaceProtocol: 42)))
            }
            .descriptors()
            .map(|setting| setting.protocol_code())
            .collect::<Vec<_>>()
        );
    }

    #[test]
    fn it_has_description_string_index() {
        assert_eq!(
            vec!(Some(42)),
            unsafe { super::from_libusb(&interface!(interface_descriptor!(iInterface: 42))) }
                .descriptors()
                .map(|setting| setting.description_string_index())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn it_handles_missing_description_string_index() {
        assert_eq!(
            vec!(None),
            unsafe { super::from_libusb(&interface!(interface_descriptor!(iInterface: 0))) }
                .descriptors()
                .map(|setting| setting.description_string_index())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn it_has_num_endpoints() {
        let endpoint1 = endpoint_descriptor!(bEndpointAddress: 0x81);
        let endpoint2 = endpoint_descriptor!(bEndpointAddress: 0x01);

        assert_eq!(
            vec!(2),
            unsafe { super::from_libusb(&interface!(interface_descriptor!(endpoint1, endpoint2))) }
                .descriptors()
                .map(|setting| setting.num_endpoints())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn it_has_endpoints() {
        let libusb_interface = interface!(interface_descriptor!(
            endpoint_descriptor!(bEndpointAddress: 0x87)
        ));
        let interface = unsafe { super::from_libusb(&libusb_interface) };

        let endpoint_addresses = interface
            .descriptors()
            .next()
            .unwrap()
            .endpoint_descriptors()
            .map(|endpoint| endpoint.address())
            .collect::<Vec<_>>();

        assert_eq!(vec![0x87], endpoint_addresses);
    }
}
