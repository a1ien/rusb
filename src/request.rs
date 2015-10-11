/// An endpoint's direction.
#[derive(Debug,PartialEq)]
pub enum Direction {
    /// An input endpoint (device to host).
    In,

    /// An output endpoint (host to device).
    Out
}

/// A control transfer's request type.
#[derive(Debug,PartialEq)]
pub enum RequestType {
    /// Standard requests
    Standard ,

    /// Class requests
    Class,

    /// Vendor requests
    Vendor,

    /// Reserved
    Reserved,
}

/// A control transfer's recipient.
#[derive(Debug,PartialEq)]
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

/// Describes a control request.
#[derive(Debug,PartialEq)]
pub struct ControlRequest {
    direction:    Direction,
    request_type: RequestType,
    recipient:    Recipient,
}

impl ControlRequest {
    pub fn new(direction: Direction,
               request_type: RequestType,
               recipient: Recipient) -> ControlRequest {
                   ControlRequest {
                       direction:     direction,
                       request_type:  request_type,
                       recipient:     recipient,
                   }
               }

    /// Returns the ControlRequest as a `bmRequestType` field.
    pub fn to_u8(&self) -> u8 {
        let a: u8 = match self.direction {
            Direction::Out      => ::ffi::LIBUSB_ENDPOINT_OUT,
            Direction::In       => ::ffi::LIBUSB_ENDPOINT_IN
        };
        let b: u8 = match self.request_type {
            RequestType::Standard      => ::ffi::LIBUSB_REQUEST_TYPE_STANDARD,
            RequestType::Class         => ::ffi::LIBUSB_REQUEST_TYPE_CLASS,
            RequestType::Vendor        => ::ffi::LIBUSB_REQUEST_TYPE_VENDOR,
            RequestType::Reserved      => ::ffi::LIBUSB_REQUEST_TYPE_RESERVED
        };
        let c: u8 = match self.recipient {
            Recipient::Device     => ::ffi::LIBUSB_RECIPIENT_DEVICE,
            Recipient::Interface  => ::ffi::LIBUSB_RECIPIENT_INTERFACE,
            Recipient::Endpoint   => ::ffi::LIBUSB_RECIPIENT_ENDPOINT,
            Recipient::Other      => ::ffi::LIBUSB_RECIPIENT_OTHER
        };
        a | b | c
    }
}
