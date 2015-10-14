/// Transfer and endpoint directions.
#[derive(Debug,PartialEq,Eq,Clone,Copy,Hash)]
pub enum Direction {
    /// Direction for read (device to host) transfers.
    In,

    /// Direction for write (host to device) transfers.
    Out
}

/// Types of control transfers.
#[derive(Debug,PartialEq,Eq,Clone,Copy,Hash)]
pub enum RequestType {
    /// Requests that are defined by the USB standard.
    Standard,

    /// Requests that are defined by a device class, e.g., HID.
    Class,

    /// Vendor-specific requests.
    Vendor,

    /// Reserved for future use.
    Reserved,
}

/// Recipients of control transfers.
#[derive(Debug,PartialEq,Eq,Clone,Copy,Hash)]
pub enum Recipient {
    /// The recipient is a device.
    Device,

    /// The recipient is an interface.
    Interface,

    /// The recipient is an endpoint.
    Endpoint,

    /// Other.
    Other,
}

/// Builds a value for the `bmRequestType` field of a control transfer setup packet.
///
/// The `bmRequestType` field of a USB control transfer setup packet is a bit field specifying
/// three parameters, which are given to this function by corresponding enum values.
///
/// ## Examples
///
/// The following example returns a `bmRequestType` value for a standard inbound transfer from the
/// device, which could be used for reading a device's descriptors:
///
/// ```no_run
/// use libusb::{Direction,RequestType,Recipient};
///
/// libusb::request_type(Direction::In, RequestType::Standard, Recipient::Device);
/// ```
pub fn request_type(direction: Direction, request_type: RequestType, recipient: Recipient) -> u8 {
    let mut value: u8 = match direction {
        Direction::Out => ::libusb::LIBUSB_ENDPOINT_OUT,
        Direction::In  => ::libusb::LIBUSB_ENDPOINT_IN,
    };

    value |= match request_type {
        RequestType::Standard => ::libusb::LIBUSB_REQUEST_TYPE_STANDARD,
        RequestType::Class    => ::libusb::LIBUSB_REQUEST_TYPE_CLASS,
        RequestType::Vendor   => ::libusb::LIBUSB_REQUEST_TYPE_VENDOR,
        RequestType::Reserved => ::libusb::LIBUSB_REQUEST_TYPE_RESERVED,
    };

    value |= match recipient {
        Recipient::Device    => ::libusb::LIBUSB_RECIPIENT_DEVICE,
        Recipient::Interface => ::libusb::LIBUSB_RECIPIENT_INTERFACE,
        Recipient::Endpoint  => ::libusb::LIBUSB_RECIPIENT_ENDPOINT,
        Recipient::Other     => ::libusb::LIBUSB_RECIPIENT_OTHER,
    };

    value
}

#[cfg(test)]
mod test {
    use super::*;

    // direction

    #[test]
    fn it_builds_request_type_for_out_direction() {
        assert_eq!(request_type(Direction::Out, RequestType::Standard, Recipient::Device) & 0x80, 0x00);
    }

    #[test]
    fn it_builds_request_type_for_in_direction() {
        assert_eq!(request_type(Direction::In, RequestType::Standard, Recipient::Device) & 0x80, 0x80);
    }

    // request type

    #[test]
    fn it_builds_request_type_for_standard_request() {
        assert_eq!(request_type(Direction::Out, RequestType::Standard, Recipient::Device) & 0x60, 0x00);
    }

    #[test]
    fn it_builds_request_type_for_class_request() {
        assert_eq!(request_type(Direction::Out, RequestType::Class, Recipient::Device) & 0x60, 0x20);
    }

    #[test]
    fn it_builds_request_type_for_vendor_request() {
        assert_eq!(request_type(Direction::Out, RequestType::Vendor, Recipient::Device) & 0x60, 0x40);
    }

    #[test]
    fn it_builds_request_type_for_reserved_request() {
        assert_eq!(request_type(Direction::Out, RequestType::Reserved, Recipient::Device) & 0x60, 0x60);
    }

    // recipient

    #[test]
    fn it_builds_request_type_for_device_recipient() {
        assert_eq!(request_type(Direction::Out, RequestType::Standard, Recipient::Device) & 0x0F, 0x00);
    }

    #[test]
    fn it_builds_request_type_for_interface_recipient() {
        assert_eq!(request_type(Direction::Out, RequestType::Standard, Recipient::Interface) & 0x0F, 0x01);
    }

    #[test]
    fn it_builds_request_type_for_endpoint_recipient() {
        assert_eq!(request_type(Direction::Out, RequestType::Standard, Recipient::Endpoint) & 0x0F, 0x02);
    }

    #[test]
    fn it_builds_request_type_for_other_recipient() {
        assert_eq!(request_type(Direction::Out, RequestType::Standard, Recipient::Other) & 0x0F, 0x03);
    }
}
