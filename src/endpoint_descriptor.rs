use ::fields::{Direction,TransferType,SyncType,UsageType};

/// Describes an endpoint.
pub struct EndpointDescriptor<'a> {
    descriptor: &'a ::libusb::libusb_endpoint_descriptor,
}

impl<'a> EndpointDescriptor<'a> {
    /// Returns the endpoint's address.
    pub fn address(&self) -> u8 {
        self.descriptor.bEndpointAddress
    }

    /// Returns the endpoint number.
    pub fn number(&self) -> u8 {
        self.descriptor.bEndpointAddress & 0x07
    }

    /// Returns the endpoint's direction.
    pub fn direction(&self) -> Direction {
        match self.descriptor.bEndpointAddress & ::libusb::LIBUSB_ENDPOINT_DIR_MASK {
            ::libusb::LIBUSB_ENDPOINT_OUT    => Direction::Out,
            ::libusb::LIBUSB_ENDPOINT_IN | _ => Direction::In
        }
    }

    /// Returns the endpoint's transfer type.
    pub fn transfer_type(&self) -> TransferType {
        match self.descriptor.bmAttributes & ::libusb::LIBUSB_TRANSFER_TYPE_MASK {
            ::libusb::LIBUSB_TRANSFER_TYPE_CONTROL       => TransferType::Control,
            ::libusb::LIBUSB_TRANSFER_TYPE_ISOCHRONOUS   => TransferType::Isochronous,
            ::libusb::LIBUSB_TRANSFER_TYPE_BULK          => TransferType::Bulk,
            ::libusb::LIBUSB_TRANSFER_TYPE_INTERRUPT | _ => TransferType::Interrupt
        }
    }

    /// Returns the endpoint's synchronisation mode.
    ///
    /// The return value of this method is only valid for isochronous endpoints.
    pub fn sync_type(&self) -> SyncType {
        match (self.descriptor.bmAttributes & ::libusb::LIBUSB_ISO_SYNC_TYPE_MASK) >> 2 {
            ::libusb::LIBUSB_ISO_SYNC_TYPE_NONE     => SyncType::NoSync,
            ::libusb::LIBUSB_ISO_SYNC_TYPE_ASYNC    => SyncType::Asynchronous,
            ::libusb::LIBUSB_ISO_SYNC_TYPE_ADAPTIVE => SyncType::Adaptive,
            ::libusb::LIBUSB_ISO_SYNC_TYPE_SYNC | _ => SyncType::Synchronous
        }
    }

    /// Returns the endpoint's usage type.
    ///
    /// The return value of this method is only valid for isochronous endpoints.
    pub fn usage_type(&self) -> UsageType {
        match (self.descriptor.bmAttributes & ::libusb::LIBUSB_ISO_USAGE_TYPE_MASK) >> 4 {
            ::libusb::LIBUSB_ISO_USAGE_TYPE_DATA     => UsageType::Data,
            ::libusb::LIBUSB_ISO_USAGE_TYPE_FEEDBACK => UsageType::Feedback,
            ::libusb::LIBUSB_ISO_USAGE_TYPE_IMPLICIT => UsageType::FeedbackData,
            _                                        => UsageType::Reserved
        }
    }

    /// Returns the endpoint's maximum packet size.
    pub fn max_packet_size(&self) -> u16 {
        self.descriptor.wMaxPacketSize
    }

    /// Returns the endpoint's polling interval.
    pub fn interval(&self) -> u8 {
        self.descriptor.bInterval
    }
}


#[doc(hidden)]
pub fn from_libusb(endpoint: &::libusb::libusb_endpoint_descriptor) -> EndpointDescriptor {
    EndpointDescriptor { descriptor: endpoint }
}


#[cfg(test)]
mod test {
    use ::fields::{Direction,TransferType,SyncType,UsageType};

    #[test]
    fn it_interprets_number_for_output_endpoints() {
        assert_eq!(0, super::from_libusb(&endpoint_descriptor!(bEndpointAddress: 0b0000_0000)).number());
        assert_eq!(1, super::from_libusb(&endpoint_descriptor!(bEndpointAddress: 0b0000_0001)).number());
    }

    #[test]
    fn it_interprets_number_for_input_endpoints() {
        assert_eq!(2, super::from_libusb(&endpoint_descriptor!(bEndpointAddress: 0b1000_0010)).number());
        assert_eq!(3, super::from_libusb(&endpoint_descriptor!(bEndpointAddress: 0b1000_0011)).number());
    }

    #[test]
    fn it_ignores_reserved_bits_in_address() {
        assert_eq!(0, super::from_libusb(&endpoint_descriptor!(bEndpointAddress: 0b0000_1000)).number());
        assert_eq!(0, super::from_libusb(&endpoint_descriptor!(bEndpointAddress: 0b0001_0000)).number());
        assert_eq!(0, super::from_libusb(&endpoint_descriptor!(bEndpointAddress: 0b0010_0000)).number());
        assert_eq!(0, super::from_libusb(&endpoint_descriptor!(bEndpointAddress: 0b0100_0000)).number());
        assert_eq!(7, super::from_libusb(&endpoint_descriptor!(bEndpointAddress: 0b1111_1111)).number());
    }

    #[test]
    fn it_interprets_direction_bit_in_address() {
        assert_eq!(Direction::Out, super::from_libusb(&endpoint_descriptor!(bEndpointAddress: 0b0000_0000)).direction());
        assert_eq!(Direction::In,  super::from_libusb(&endpoint_descriptor!(bEndpointAddress: 0b1000_0000)).direction());
    }

    #[test]
    fn it_interprets_transfer_type_in_attributes() {
        assert_eq!(TransferType::Control,     super::from_libusb(&endpoint_descriptor!(bmAttributes: 0b0000_0000)).transfer_type());
        assert_eq!(TransferType::Isochronous, super::from_libusb(&endpoint_descriptor!(bmAttributes: 0b0000_0001)).transfer_type());
        assert_eq!(TransferType::Bulk,        super::from_libusb(&endpoint_descriptor!(bmAttributes: 0b0000_0010)).transfer_type());
        assert_eq!(TransferType::Interrupt,   super::from_libusb(&endpoint_descriptor!(bmAttributes: 0b0000_0011)).transfer_type());
    }

    #[test]
    fn it_interprets_synchronization_type_in_attributes() {
        assert_eq!(SyncType::NoSync,       super::from_libusb(&endpoint_descriptor!(bmAttributes: 0b0000_0001)).sync_type());
        assert_eq!(SyncType::Asynchronous, super::from_libusb(&endpoint_descriptor!(bmAttributes: 0b0000_0101)).sync_type());
        assert_eq!(SyncType::Adaptive,     super::from_libusb(&endpoint_descriptor!(bmAttributes: 0b0000_1001)).sync_type());
        assert_eq!(SyncType::Synchronous,  super::from_libusb(&endpoint_descriptor!(bmAttributes: 0b0000_1101)).sync_type());
    }

    #[test]
    fn it_interprets_usage_type_in_attributes() {
        assert_eq!(UsageType::Data,         super::from_libusb(&endpoint_descriptor!(bmAttributes: 0b0000_0001)).usage_type());
        assert_eq!(UsageType::Feedback,     super::from_libusb(&endpoint_descriptor!(bmAttributes: 0b0001_0001)).usage_type());
        assert_eq!(UsageType::FeedbackData, super::from_libusb(&endpoint_descriptor!(bmAttributes: 0b0010_0001)).usage_type());
        assert_eq!(UsageType::Reserved,     super::from_libusb(&endpoint_descriptor!(bmAttributes: 0b0011_0001)).usage_type());
    }

    #[test]
    fn it_has_max_packet_size() {
        assert_eq!(64,    super::from_libusb(&endpoint_descriptor!(wMaxPacketSize: 64)).max_packet_size());
        assert_eq!(4096,  super::from_libusb(&endpoint_descriptor!(wMaxPacketSize: 4096)).max_packet_size());
        assert_eq!(65535, super::from_libusb(&endpoint_descriptor!(wMaxPacketSize: 65535)).max_packet_size());
    }

    #[test]
    fn it_has_interval() {
        assert_eq!(1,   super::from_libusb(&endpoint_descriptor!(bInterval: 1)).interval());
        assert_eq!(20,  super::from_libusb(&endpoint_descriptor!(bInterval: 20)).interval());
        assert_eq!(255, super::from_libusb(&endpoint_descriptor!(bInterval: 255)).interval());
    }
}
