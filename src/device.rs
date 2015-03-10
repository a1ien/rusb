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
  //manufacturer_index: u8,
  //product_index: u8,
  //serial_number_index: u8,
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
    self.configurations.as_slice()
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
    //manufacturer_index:  device.iManufacturer,
    //product_index:       device.iProduct,
    //serial_number_index: device.iSerialNumber,
    configurations:   configs
  }
}


#[cfg(test)]
mod test {
  extern crate rand;

  use super::Speed;

  #[test]
  fn it_has_bus_number() {
    let n = rand::random();
    assert_eq!(n, ::device::from_libusb(&device_descriptor!(bDeviceClass: 0), vec![], n, 0, 0).bus_number());
  }

  #[test]
  fn it_has_address() {
    let n = rand::random();
    assert_eq!(n, ::device::from_libusb(&device_descriptor!(bDeviceClass: 0), vec![], 0, n, 0).address());
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
    let v1 = rand::random() % 10;
    let v2 = rand::random() % 10;
    let v3 = rand::random() % 10;
    let v4 = rand::random() % 10;

    let v: u16 = v1 << 12 | v2 << 8 | v3 << 4 | v4;
    assert_eq!(::Version::from_bcd(v), ::device::from_libusb(&device_descriptor!(bcdUSB: v), vec![], 0, 0, 0).usb_version());
  }

  #[test]
  fn it_has_device_version() {
    let v1 = rand::random() % 10;
    let v2 = rand::random() % 10;
    let v3 = rand::random() % 10;
    let v4 = rand::random() % 10;

    let v: u16 = v1 << 12 | v2 << 8 | v3 << 4 | v4;
    assert_eq!(::Version::from_bcd(v), ::device::from_libusb(&device_descriptor!(bcdDevice: v), vec![], 0, 0, 0).device_version());
  }

  #[test]
  fn it_has_class_code() {
    let n = rand::random();
    assert_eq!(n, ::device::from_libusb(&device_descriptor!(bDeviceClass: n), vec![], 0, 0, 0).class_code());
  }

  #[test]
  fn it_has_sub_class_code() {
    let n = rand::random();
    assert_eq!(n, ::device::from_libusb(&device_descriptor!(bDeviceSubClass: n), vec![], 0, 0, 0).sub_class_code());
  }

  #[test]
  fn it_has_protocol_code() {
    let n = rand::random();
    assert_eq!(n, ::device::from_libusb(&device_descriptor!(bDeviceProtocol: n), vec![], 0, 0, 0).protocol_code());
  }

  #[test]
  fn it_has_vendor_id() {
    let n = rand::random();
    assert_eq!(n, ::device::from_libusb(&device_descriptor!(idVendor: n), vec![], 0, 0, 0).vendor_id());
  }

  #[test]
  fn it_has_product_id() {
    let n = rand::random();
    assert_eq!(n, ::device::from_libusb(&device_descriptor!(idProduct: n), vec![], 0, 0, 0).product_id());
  }

  #[test]
  fn it_has_max_packet_size() {
    let n = rand::random();
    assert_eq!(n, ::device::from_libusb(&device_descriptor!(bMaxPacketSize0: n), vec![], 0, 0, 0).max_packet_size());
  }
}
