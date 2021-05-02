use libc::c_int;
use libusb1_sys::constants::*;

/// Device speeds. Indicates the speed at which a device is operating.
/// - [libusb_supported_speed](http://libusb.sourceforge.net/api-1.0/group__libusb__dev.html#ga1454797ecc0de4d084c1619c420014f6)
/// - [USB release versions](https://en.wikipedia.org/wiki/USB#Release_versions)
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Speed {
    /// The operating system doesn't know the device speed.
    Unknown,

    /// The device is operating at low speed (1.5 Mbps).
    Low,

    /// The device is operating at full speed (12 Mbps).
    Full,

    /// The device is operating at high speed (480 Mbps).
    High,

    /// The device is operating at super speed (5 Gbps).
    Super,
}

#[doc(hidden)]
pub(crate) fn speed_from_libusb(n: c_int) -> Speed {
    match n {
        LIBUSB_SPEED_SUPER => Speed::Super,
        LIBUSB_SPEED_HIGH => Speed::High,
        LIBUSB_SPEED_FULL => Speed::Full,
        LIBUSB_SPEED_LOW => Speed::Low,

        LIBUSB_SPEED_UNKNOWN | _ => Speed::Unknown,
    }
}

/// Transfer and endpoint directions.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Direction {
    /// Direction for read (device to host) transfers.
    In,

    /// Direction for write (host to device) transfers.
    Out,
}

/// An endpoint's transfer type.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum TransferType {
    /// Control endpoint.
    Control,

    /// Isochronous endpoint.
    Isochronous,

    /// Bulk endpoint.
    Bulk,

    /// Interrupt endpoint.
    Interrupt,
}

/// Isochronous synchronization mode.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum SyncType {
    /// No synchronisation.
    NoSync,

    /// Asynchronous.
    Asynchronous,

    /// Adaptive.
    Adaptive,

    /// Synchronous.
    Synchronous,
}

/// Isochronous usage type.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum UsageType {
    /// Data endpoint.
    Data,

    /// Feedback endpoint.
    Feedback,

    /// Explicit feedback data endpoint.
    FeedbackData,

    /// Reserved.
    Reserved,
}

/// Types of control transfers.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
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
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
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

/// A three-part version consisting of major, minor, and sub minor components.
///
/// This can be used to represent versions of the format `J.M.N`, where `J` is the major version,
/// `M` is the minor version, and `N` is the sub minor version. A version is constructed by
/// providing the fields in the same order to the tuple. For example:
///
/// ```
/// rusb::Version(0, 2, 1);
/// ```
///
/// represents the version 0.2.1.
///
/// The intended use case of `Version` is to extract meaning from the version fields in USB
/// descriptors, such as `bcdUSB` and `bcdDevice` in device descriptors.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Version(pub u8, pub u8, pub u8);

impl Version {
    /// Extracts a version from a binary coded decimal (BCD) field. BCD fields exist in USB
    /// descriptors as 16-bit integers encoding a version as `0xJJMN`, where `JJ` is the major
    /// version, `M` is the minor version, and `N` is the sub minor version. For example, 2.0 is
    /// endoded as `0x0200` and 1.1 is encoded as `0x0110`.
    pub fn from_bcd(mut raw: u16) -> Self {
        let sub_minor: u8 = (raw & 0x000F) as u8;
        raw >>= 4;

        let minor: u8 = (raw & 0x000F) as u8;
        raw >>= 4;

        let mut major: u8 = (raw & 0x000F) as u8;
        raw >>= 4;

        major += (10 * raw) as u8;

        Version(major, minor, sub_minor)
    }

    /// Returns the major version.
    pub fn major(self) -> u8 {
        let Version(major, _, _) = self;
        major
    }

    /// Returns the minor version.
    pub fn minor(self) -> u8 {
        let Version(_, minor, _) = self;
        minor
    }

    /// Returns the sub minor version.
    pub fn sub_minor(self) -> u8 {
        let Version(_, _, sub_minor) = self;
        sub_minor
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major(), self.minor(), self.sub_minor())
    }
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
/// use rusb::{Direction,RequestType,Recipient};
///
/// rusb::request_type(Direction::In, RequestType::Standard, Recipient::Device);
/// ```
pub fn request_type(direction: Direction, request_type: RequestType, recipient: Recipient) -> u8 {
    let mut value: u8 = match direction {
        Direction::Out => LIBUSB_ENDPOINT_OUT,
        Direction::In => LIBUSB_ENDPOINT_IN,
    };

    value |= match request_type {
        RequestType::Standard => LIBUSB_REQUEST_TYPE_STANDARD,
        RequestType::Class => LIBUSB_REQUEST_TYPE_CLASS,
        RequestType::Vendor => LIBUSB_REQUEST_TYPE_VENDOR,
        RequestType::Reserved => LIBUSB_REQUEST_TYPE_RESERVED,
    };

    value |= match recipient {
        Recipient::Device => LIBUSB_RECIPIENT_DEVICE,
        Recipient::Interface => LIBUSB_RECIPIENT_INTERFACE,
        Recipient::Endpoint => LIBUSB_RECIPIENT_ENDPOINT,
        Recipient::Other => LIBUSB_RECIPIENT_OTHER,
    };

    value
}

#[cfg(test)]
mod test {
    use super::*;

    // Version

    #[test]
    fn version_returns_major_version() {
        assert_eq!(1, Version(1, 0, 0).major());
        assert_eq!(2, Version(2, 0, 0).major());
    }

    #[test]
    fn version_returns_minor_version() {
        assert_eq!(1, Version(0, 1, 0).minor());
        assert_eq!(2, Version(0, 2, 0).minor());
    }

    #[test]
    fn version_returns_sub_minor_version() {
        assert_eq!(1, Version(0, 0, 1).sub_minor());
        assert_eq!(2, Version(0, 0, 2).sub_minor());
    }

    #[test]
    fn version_parses_major_version() {
        assert_eq!(3, Version::from_bcd(0x0300).major());
    }

    #[test]
    fn version_parses_long_major_version() {
        assert_eq!(12, Version::from_bcd(0x1200).major());
    }

    #[test]
    fn version_parses_minor_version() {
        assert_eq!(1, Version::from_bcd(0x0010).minor());
        assert_eq!(2, Version::from_bcd(0x0020).minor());
    }

    #[test]
    fn version_parses_sub_minor_version() {
        assert_eq!(1, Version::from_bcd(0x0001).sub_minor());
        assert_eq!(2, Version::from_bcd(0x0002).sub_minor());
    }

    #[test]
    fn version_parses_full_version() {
        assert_eq!(Version(12, 3, 4), Version::from_bcd(0x1234));
    }

    #[test]
    fn version_display() {
        assert_eq!(Version(2, 45, 13).to_string(), "2.45.13");
    }

    // request_type for direction

    #[test]
    fn request_type_builds_value_for_out_direction() {
        assert_eq!(
            request_type(Direction::Out, RequestType::Standard, Recipient::Device) & 0x80,
            0x00
        );
    }

    #[test]
    fn request_type_builds_value_for_in_direction() {
        assert_eq!(
            request_type(Direction::In, RequestType::Standard, Recipient::Device) & 0x80,
            0x80
        );
    }

    // request_type for request type

    #[test]
    fn request_type_builds_value_for_standard_request() {
        assert_eq!(
            request_type(Direction::Out, RequestType::Standard, Recipient::Device) & 0x60,
            0x00
        );
    }

    #[test]
    fn request_type_builds_value_for_class_request() {
        assert_eq!(
            request_type(Direction::Out, RequestType::Class, Recipient::Device) & 0x60,
            0x20
        );
    }

    #[test]
    fn request_type_builds_value_for_vendor_request() {
        assert_eq!(
            request_type(Direction::Out, RequestType::Vendor, Recipient::Device) & 0x60,
            0x40
        );
    }

    #[test]
    fn request_type_builds_value_for_reserved_request() {
        assert_eq!(
            request_type(Direction::Out, RequestType::Reserved, Recipient::Device) & 0x60,
            0x60
        );
    }

    // request_type for recipient

    #[test]
    fn request_type_builds_value_for_device_recipient() {
        assert_eq!(
            request_type(Direction::Out, RequestType::Standard, Recipient::Device) & 0x0F,
            0x00
        );
    }

    #[test]
    fn request_type_builds_value_for_interface_recipient() {
        assert_eq!(
            request_type(Direction::Out, RequestType::Standard, Recipient::Interface) & 0x0F,
            0x01
        );
    }

    #[test]
    fn request_type_builds_value_for_endpoint_recipient() {
        assert_eq!(
            request_type(Direction::Out, RequestType::Standard, Recipient::Endpoint) & 0x0F,
            0x02
        );
    }

    #[test]
    fn request_type_builds_value_for_other_recipient() {
        assert_eq!(
            request_type(Direction::Out, RequestType::Standard, Recipient::Other) & 0x0F,
            0x03
        );
    }
}
