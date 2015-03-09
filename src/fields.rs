/// A three-part version consisting of major, minor, and sub minor components.
///
/// This can be used to represent versions of the format `J.M.N`, where `J` is the major version,
/// `M` is the minor version, and `N` is the sub minor version. A version is constructed by
/// providing the fields in the same order to the tuple. For example:
///
/// ```ignore
/// libusb::Version(0, 2, 1);
/// ```
///
/// represents the version 0.2.1.
///
/// The intended use case of `Version` is to extract meaning from the version fields in USB
/// descriptors, such as `bcdUSB` and `bcdDevice` in device descriptors.
#[derive(Debug,PartialEq)]
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
  pub fn major(&self) -> u8 {
    let Version(major, _, _) = *self;
    major
  }

  /// Returns the minor version.
  pub fn minor(&self) -> u8 {
    let Version(_, minor, _) = *self;
    minor
  }

  /// Returns the sub minor version.
  pub fn sub_minor(&self) -> u8 {
    let Version(_, _, sub_minor) = *self;
    sub_minor
  }
}


#[cfg(test)]
mod version_test {
  use ::Version;

  #[test]
  fn it_returns_major_version() {
    assert_eq!(1, Version(1, 0, 0).major());
    assert_eq!(2, Version(2, 0, 0).major());
  }

  #[test]
  fn it_returns_minor_version() {
    assert_eq!(1, Version(0, 1, 0).minor());
    assert_eq!(2, Version(0, 2, 0).minor());
  }

  #[test]
  fn it_returns_sub_minor_version() {
    assert_eq!(1, Version(0, 0, 1).sub_minor());
    assert_eq!(2, Version(0, 0, 2).sub_minor());
  }

  #[test]
  fn it_parses_major_version() {
    assert_eq!(3, Version::from_bcd(0x0300).major());
  }

  #[test]
  fn it_parses_long_major_version() {
    assert_eq!(12, Version::from_bcd(0x1200).major());
  }

  #[test]
  fn it_parses_minor_version() {
    assert_eq!(1, Version::from_bcd(0x0010).minor());
    assert_eq!(2, Version::from_bcd(0x0020).minor());
  }

  #[test]
  fn it_parses_sub_minor_version() {
    assert_eq!(1, Version::from_bcd(0x0001).sub_minor());
    assert_eq!(2, Version::from_bcd(0x0002).sub_minor());
  }

  #[test]
  fn it_parses_full_version() {
    assert_eq!(Version(12, 3, 4), Version::from_bcd(0x1234));
  }
}
