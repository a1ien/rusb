/// An endpoint's direction.
#[derive(Debug,PartialEq)]
pub enum Direction {
  /// An input endpoint (device to host).
  In,

  /// An output endpoint (host to device).
  Out
}


/// An endpoint's transfer type.
#[derive(Debug,PartialEq)]
pub enum TransferType {
  /// Control endpoint.
  Control,

  /// Isochronous endpoint.
  Isochronous,

  /// Bulk endpoint.
  Bulk,

  /// Interrupt endpoint.
  Interrupt
}


/// Isochronous synchronization mode.
#[derive(Debug,PartialEq)]
pub enum SyncType {
  /// No synchronisation.
  NoSync,

  /// Asynchronous.
  Asynchronous,

  /// Adaptive.
  Adaptive,

  /// Synchronous.
  Synchronous
}


/// Isochronous usage type.
#[derive(Debug,PartialEq)]
pub enum UsageType {
  /// Data endpoint.
  Data,

  /// Feedback endpoint.
  Feedback,

  /// Explicit feedback data endpoint.
  FeedbackData,

  /// Reserved.
  Reserved
}


/// Describes an endpoint.
#[derive(Debug,PartialEq)]
pub struct Endpoint {
  address: u8,
  attributes: u8,
  max_packet_size: u16,
  interval: u8
}

impl Endpoint {
  /// Returns the endpoint's address.
  pub fn address(&self) -> u8 {
    self.address
  }

  /// Returns the endpoint number.
  pub fn number(&self) -> u8 {
    self.address & 0x07
  }

  /// Returns the endpoint's direction.
  pub fn direction(&self) -> Direction {
    match self.address & ::ffi::LIBUSB_ENDPOINT_DIR_MASK {
      ::ffi::LIBUSB_ENDPOINT_OUT    => Direction::Out,
      ::ffi::LIBUSB_ENDPOINT_IN | _ => Direction::In
    }
  }

  /// Returns the endpoint's transfer type.
  pub fn transfer_type(&self) -> TransferType {
    match self.attributes & ::ffi::LIBUSB_TRANSFER_TYPE_MASK {
      ::ffi::LIBUSB_TRANSFER_TYPE_CONTROL       => TransferType::Control,
      ::ffi::LIBUSB_TRANSFER_TYPE_ISOCHRONOUS   => TransferType::Isochronous,
      ::ffi::LIBUSB_TRANSFER_TYPE_BULK          => TransferType::Bulk,
      ::ffi::LIBUSB_TRANSFER_TYPE_INTERRUPT | _ => TransferType::Interrupt
    }
  }

  /// Returns the endpoint's synchronisation mode.
  ///
  /// The return value of this method is only valid for isochronous endpoints.
  pub fn sync_type(&self) -> SyncType {
    match (self.attributes & ::ffi::LIBUSB_ISO_SYNC_TYPE_MASK) >> 2 {
      ::ffi::LIBUSB_ISO_SYNC_TYPE_NONE     => SyncType::NoSync,
      ::ffi::LIBUSB_ISO_SYNC_TYPE_ASYNC    => SyncType::Asynchronous,
      ::ffi::LIBUSB_ISO_SYNC_TYPE_ADAPTIVE => SyncType::Adaptive,
      ::ffi::LIBUSB_ISO_SYNC_TYPE_SYNC | _ => SyncType::Synchronous
    }
  }

  /// Returns the endpoint's usage type.
  ///
  /// The return value of this method is only valid for isochronous endpoints.
  pub fn usage_type(&self) -> UsageType {
    match (self.attributes & ::ffi::LIBUSB_ISO_USAGE_TYPE_MASK) >> 4 {
      ::ffi::LIBUSB_ISO_USAGE_TYPE_DATA     => UsageType::Data,
      ::ffi::LIBUSB_ISO_USAGE_TYPE_FEEDBACK => UsageType::Feedback,
      ::ffi::LIBUSB_ISO_USAGE_TYPE_IMPLICIT => UsageType::FeedbackData,
      _                                     => UsageType::Reserved
    }
  }

  /// Returns the endpoint's maximum packet size.
  pub fn max_packet_size(&self) -> u16 {
    self.max_packet_size
  }

  /// Returns the endpoint's polling interval.
  pub fn interval(&self) -> u8 {
    self.interval
  }
}


// Not exported outside the crate.
pub fn from_libusb(endpoint: &::ffi::libusb_endpoint_descriptor) -> Endpoint {
  Endpoint {
    address:         endpoint.bEndpointAddress,
    attributes:      endpoint.bmAttributes,
    max_packet_size: endpoint.wMaxPacketSize,
    interval:        endpoint.bInterval
  }
}


#[cfg(test)]
mod test {
  use super::{Direction,TransferType,SyncType,UsageType};

  #[test]
  fn it_interprets_number_for_output_endpoints() {
    assert_eq!(0, ::endpoint::from_libusb(&endpoint_descriptor!(bEndpointAddress: 0b0000_0000)).number());
    assert_eq!(1, ::endpoint::from_libusb(&endpoint_descriptor!(bEndpointAddress: 0b0000_0001)).number());
  }

  #[test]
  fn it_interprets_number_for_input_endpoints() {
    assert_eq!(2, ::endpoint::from_libusb(&endpoint_descriptor!(bEndpointAddress: 0b1000_0010)).number());
    assert_eq!(3, ::endpoint::from_libusb(&endpoint_descriptor!(bEndpointAddress: 0b1000_0011)).number());
  }

  #[test]
  fn it_ignores_reserved_bits_in_address() {
    assert_eq!(0, ::endpoint::from_libusb(&endpoint_descriptor!(bEndpointAddress: 0b0000_1000)).number());
    assert_eq!(0, ::endpoint::from_libusb(&endpoint_descriptor!(bEndpointAddress: 0b0001_0000)).number());
    assert_eq!(0, ::endpoint::from_libusb(&endpoint_descriptor!(bEndpointAddress: 0b0010_0000)).number());
    assert_eq!(0, ::endpoint::from_libusb(&endpoint_descriptor!(bEndpointAddress: 0b0100_0000)).number());
    assert_eq!(7, ::endpoint::from_libusb(&endpoint_descriptor!(bEndpointAddress: 0b1111_1111)).number());
  }

  #[test]
  fn it_interprets_direction_bit_in_address() {
    assert_eq!(Direction::Out, ::endpoint::from_libusb(&endpoint_descriptor!(bEndpointAddress: 0b0000_0000)).direction());
    assert_eq!(Direction::In,  ::endpoint::from_libusb(&endpoint_descriptor!(bEndpointAddress: 0b1000_0000)).direction());
  }

  #[test]
  fn it_interprets_transfer_type_in_attributes() {
    assert_eq!(TransferType::Control,     ::endpoint::from_libusb(&endpoint_descriptor!(bmAttributes: 0b0000_0000)).transfer_type());
    assert_eq!(TransferType::Isochronous, ::endpoint::from_libusb(&endpoint_descriptor!(bmAttributes: 0b0000_0001)).transfer_type());
    assert_eq!(TransferType::Bulk,        ::endpoint::from_libusb(&endpoint_descriptor!(bmAttributes: 0b0000_0010)).transfer_type());
    assert_eq!(TransferType::Interrupt,   ::endpoint::from_libusb(&endpoint_descriptor!(bmAttributes: 0b0000_0011)).transfer_type());
  }

  #[test]
  fn it_interprets_synchronization_type_in_attributes() {
    assert_eq!(SyncType::NoSync,       ::endpoint::from_libusb(&endpoint_descriptor!(bmAttributes: 0b0000_0001)).sync_type());
    assert_eq!(SyncType::Asynchronous, ::endpoint::from_libusb(&endpoint_descriptor!(bmAttributes: 0b0000_0101)).sync_type());
    assert_eq!(SyncType::Adaptive,     ::endpoint::from_libusb(&endpoint_descriptor!(bmAttributes: 0b0000_1001)).sync_type());
    assert_eq!(SyncType::Synchronous,  ::endpoint::from_libusb(&endpoint_descriptor!(bmAttributes: 0b0000_1101)).sync_type());
  }

  #[test]
  fn it_interprets_usage_type_in_attributes() {
    assert_eq!(UsageType::Data,         ::endpoint::from_libusb(&endpoint_descriptor!(bmAttributes: 0b0000_0001)).usage_type());
    assert_eq!(UsageType::Feedback,     ::endpoint::from_libusb(&endpoint_descriptor!(bmAttributes: 0b0001_0001)).usage_type());
    assert_eq!(UsageType::FeedbackData, ::endpoint::from_libusb(&endpoint_descriptor!(bmAttributes: 0b0010_0001)).usage_type());
    assert_eq!(UsageType::Reserved,     ::endpoint::from_libusb(&endpoint_descriptor!(bmAttributes: 0b0011_0001)).usage_type());
  }

  #[test]
  fn it_has_max_packet_size() {
    assert_eq!(64,    ::endpoint::from_libusb(&endpoint_descriptor!(wMaxPacketSize: 64)).max_packet_size());
    assert_eq!(4096,  ::endpoint::from_libusb(&endpoint_descriptor!(wMaxPacketSize: 4096)).max_packet_size());
    assert_eq!(65535, ::endpoint::from_libusb(&endpoint_descriptor!(wMaxPacketSize: 65535)).max_packet_size());
  }

  #[test]
  fn it_has_interval() {
    assert_eq!(1,   ::endpoint::from_libusb(&endpoint_descriptor!(bInterval: 1)).interval());
    assert_eq!(20,  ::endpoint::from_libusb(&endpoint_descriptor!(bInterval: 20)).interval());
    assert_eq!(255, ::endpoint::from_libusb(&endpoint_descriptor!(bInterval: 255)).interval());
  }
}
