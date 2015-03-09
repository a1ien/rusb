use std::str;

use std::ffi::CStr;


/// A structure that describes the version of the underlying `libusb` library.
#[derive(Debug)]
pub struct LibraryVersion {
  major: u16,
  minor: u16,
  micro: u16,
  nano: u16,
  rc: &'static str,
  describe: &'static str
}

impl LibraryVersion {
  /// Library major version.
  pub fn major(&self) -> u16 {
    self.major
  }

  /// Library minor version.
  pub fn minor(&self) -> u16 {
    self.minor
  }

  /// Library micro version.
  pub fn micro(&self) -> u16 {
    self.micro
  }

  /// Library nano version.
  pub fn nano(&self) -> u16 {
    self.nano
  }

  /// Library release candidate suffix string, e.g., `"-rc4"`.
  pub fn rc(&self) -> Option<&'static str> {
    if self.rc.len() > 0 {
      Some(self.rc)
    }
    else {
      None
    }
  }
}

/// Returns a structure with the version of the running libusb library.
pub fn version() -> LibraryVersion {
  let v = unsafe { ::ffi::libusb_get_version() };

  LibraryVersion {
    major: v.major,
    minor: v.minor,
    micro: v.micro,
    nano: v.nano,
    rc: str::from_utf8(unsafe { CStr::from_ptr(v.rc) }.to_bytes()).unwrap_or(""),
    describe: str::from_utf8(unsafe { CStr::from_ptr(v.describe) }.to_bytes()).unwrap_or("")
  }
}

