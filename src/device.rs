use libc::{c_int};

use ::fields::Version;
use ::configuration::Configuration;


/// Device speeds. Indicates the speed at which a device is operating.
#[derive(Debug,PartialEq)]
pub enum Speed {
  /// The operating system doesn't know the device speed.
  Unknown,

  /// The device is operating at low speed (1.5MBps).
  Low,

  /// The device is operating at full speed (12MBps).
  Full,

  /// The device is operating at high speed (480Mps).
  High,

  /// The device is operating at super speed (5000Mbps).
  Super
}

impl Speed {
  fn from_libusb(n: c_int) -> Speed {
    match n {
      ::ffi::LIBUSB_SPEED_SUPER => Speed::Super,
      ::ffi::LIBUSB_SPEED_HIGH  => Speed::High,
      ::ffi::LIBUSB_SPEED_FULL  => Speed::Full,
      ::ffi::LIBUSB_SPEED_LOW   => Speed::Low,
      _                         => Speed::Unknown
    }
  }
}


/// Describes a device.
#[derive(Debug)]
pub struct Device {
  bus_number: u8,
  address: u8,
  device_speed: c_int,
  usb_version: u16,
  device_class: u8,
  device_sub_class: u8,
  device_protocol: u8,
  max_packet_size: u8,
  vendor_id: u16,
  product_id: u16,
  device_version: u16,
  manufacturer_index: Option<u8>,
  product_index: Option<u8>,
  serial_number_index: Option<u8>,
  configurations: Vec<Configuration>
}

impl Device {
  /// Returns the device's bus number.
  pub fn bus_number(&self) -> u8 {
    self.bus_number
  }

  /// Returns the device's address.
  pub fn address(&self) -> u8 {
    self.address
  }

  /// Returns the device's operating speed.
  pub fn speed(&self) -> Speed {
    Speed::from_libusb(self.device_speed)
  }

  /// Returns the device's maximum supported USB version.
  pub fn usb_version(&self) -> Version {
    Version::from_bcd(self.usb_version)
  }

  /// Returns the manufacturer's version of the device.
  pub fn device_version(&self) -> Version {
    Version::from_bcd(self.device_version)
  }

  /// Returns the index of the string descriptor that contains the manufacturer name.
  pub fn manufacturer_string_index(&self) -> Option<u8> {
      self.manufacturer_index
  }

  /// Returns the index of the string descriptor that contains the product name.
  pub fn product_string_index(&self) -> Option<u8> {
      self.product_index
  }

  /// Returns the index of the string descriptor that contains the device's serial number.
  pub fn serial_number_string_index(&self) -> Option<u8> {
      self.serial_number_index
  }

  /// Returns the device's class code.
  pub fn class_code(&self) -> u8 {
    self.device_class
  }

  /// Returns the device's sub class code.
  pub fn sub_class_code(&self) -> u8 {
    self.device_sub_class
  }

  /// Returns the device's protocol code.
  pub fn protocol_code(&self) -> u8 {
    self.device_protocol
  }

  /// Returns the device's vendor ID.
  pub fn vendor_id(&self) -> u16 {
    self.vendor_id
  }

  /// Returns the device's product ID.
  pub fn product_id(&self) -> u16 {
    self.product_id
  }

  /// Returns the maximum packet size of the device's first endpoint.
  pub fn max_packet_size(&self) -> u8 {
    self.max_packet_size
  }

  /// Returns a collection of the device's configurations.
  pub fn configurations(&self) -> &[Configuration] {
    &self.configurations[..]
  }
}


// Not exported outside the crate.
pub fn from_libusb(device: &::ffi::libusb_device_descriptor, configs: Vec<Configuration>, bus: u8, address: u8, speed: c_int) -> Device {
  Device {
    bus_number:       bus,
    address:          address,
    device_speed:     speed,
    usb_version:      device.bcdUSB,
    device_class:     device.bDeviceClass,
    device_sub_class: device.bDeviceSubClass,
    device_protocol:  device.bDeviceProtocol,
    max_packet_size:  device.bMaxPacketSize0,
    vendor_id:        device.idVendor,
    product_id:       device.idProduct,
    device_version:   device.bcdDevice,
    manufacturer_index: match device.iManufacturer {
      0 => None,
      n => Some(n)
    },
    product_index: match device.iProduct {
      0 => None,
      n => Some(n)
    },
    serial_number_index: match device.iSerialNumber {
      0 => None,
      n => Some(n)
    },
    configurations: configs
  }
}


#[cfg(test)]
mod test {
  use super::Speed;

  #[test]
  fn it_has_bus_number() {
    assert_eq!(42, ::device::from_libusb(&device_descriptor!(bDeviceClass: 0), vec![], 42, 0, 0).bus_number());
  }

  #[test]
  fn it_has_address() {
    assert_eq!(42, ::device::from_libusb(&device_descriptor!(bDeviceClass: 0), vec![], 0, 42, 0).address());
  }

  #[test]
  fn it_has_speed() {
    assert_eq!(Speed::Super,   ::device::from_libusb(&device_descriptor!(bDeviceClass: 0), vec![], 0, 0, ::ffi::LIBUSB_SPEED_SUPER).speed());
    assert_eq!(Speed::High,    ::device::from_libusb(&device_descriptor!(bDeviceClass: 0), vec![], 0, 0, ::ffi::LIBUSB_SPEED_HIGH).speed());
    assert_eq!(Speed::Full,    ::device::from_libusb(&device_descriptor!(bDeviceClass: 0), vec![], 0, 0, ::ffi::LIBUSB_SPEED_FULL).speed());
    assert_eq!(Speed::Low,     ::device::from_libusb(&device_descriptor!(bDeviceClass: 0), vec![], 0, 0, ::ffi::LIBUSB_SPEED_LOW).speed());
    assert_eq!(Speed::Unknown, ::device::from_libusb(&device_descriptor!(bDeviceClass: 0), vec![], 0, 0, ::ffi::LIBUSB_SPEED_UNKNOWN).speed());
    assert_eq!(Speed::Unknown, ::device::from_libusb(&device_descriptor!(bDeviceClass: 0), vec![], 0, 0, 42).speed());
  }

  #[test]
  fn it_has_usb_version() {
    assert_eq!(::Version::from_bcd(0x1234), ::device::from_libusb(&device_descriptor!(bcdUSB: 0x1234), vec![], 0, 0, 0).usb_version());
  }

  #[test]
  fn it_has_device_version() {
    assert_eq!(::Version::from_bcd(0x1234), ::device::from_libusb(&device_descriptor!(bcdDevice: 0x1234), vec![], 0, 0, 0).device_version());
  }

  #[test]
  fn it_has_manufacturer_string_index() {
    assert_eq!(Some(42), ::device::from_libusb(&device_descriptor!(iManufacturer: 42), vec![], 0, 0, 0).manufacturer_string_index());
  }

  #[test]
  fn it_handles_missing_manufacturer_string_index() {
    assert_eq!(None, ::device::from_libusb(&device_descriptor!(iManufacturer: 0), vec![], 0, 0, 0).manufacturer_string_index());
  }

  #[test]
  fn it_has_product_string_index() {
    assert_eq!(Some(42), ::device::from_libusb(&device_descriptor!(iProduct: 42), vec![], 0, 0, 0).product_string_index());
  }

  #[test]
  fn it_handles_missing_product_string_index() {
    assert_eq!(None, ::device::from_libusb(&device_descriptor!(iProduct: 0), vec![], 0, 0, 0).product_string_index());
  }

  #[test]
  fn it_has_serial_number_string_index() {
    assert_eq!(Some(42), ::device::from_libusb(&device_descriptor!(iSerialNumber: 42), vec![], 0, 0, 0).serial_number_string_index());
  }

  #[test]
  fn it_handles_missing_serial_number_string_index() {
    assert_eq!(None, ::device::from_libusb(&device_descriptor!(iSerialNumber: 0), vec![], 0, 0, 0).serial_number_string_index());
  }

  #[test]
  fn it_has_class_code() {
    assert_eq!(42, ::device::from_libusb(&device_descriptor!(bDeviceClass: 42), vec![], 0, 0, 0).class_code());
  }

  #[test]
  fn it_has_sub_class_code() {
    assert_eq!(42, ::device::from_libusb(&device_descriptor!(bDeviceSubClass: 42), vec![], 0, 0, 0).sub_class_code());
  }

  #[test]
  fn it_has_protocol_code() {
    assert_eq!(42, ::device::from_libusb(&device_descriptor!(bDeviceProtocol: 42), vec![], 0, 0, 0).protocol_code());
  }

  #[test]
  fn it_has_vendor_id() {
    assert_eq!(42, ::device::from_libusb(&device_descriptor!(idVendor: 42), vec![], 0, 0, 0).vendor_id());
  }

  #[test]
  fn it_has_product_id() {
    assert_eq!(42, ::device::from_libusb(&device_descriptor!(idProduct: 42), vec![], 0, 0, 0).product_id());
  }

  #[test]
  fn it_has_max_packet_size() {
    assert_eq!(42, ::device::from_libusb(&device_descriptor!(bMaxPacketSize0: 42), vec![], 0, 0, 0).max_packet_size());
  }
}
