use std::ffi::CStr;
use std::fmt;
use std::mem;
use std::str;

/// A structure that describes the version of the underlying `libusb` library.
pub struct LibraryVersion {
    inner: &'static ::libusb::libusb_version,
}

impl LibraryVersion {
    /// Library major version.
    pub fn major(&self) -> u16 {
        self.inner.major
    }

    /// Library minor version.
    pub fn minor(&self) -> u16 {
        self.inner.minor
    }

    /// Library micro version.
    pub fn micro(&self) -> u16 {
        self.inner.micro
    }

    /// Library nano version.
    pub fn nano(&self) -> u16 {
        self.inner.nano
    }

    /// Library release candidate suffix string, e.g., `"-rc4"`.
    pub fn rc(&self) -> Option<&'static str> {
        let cstr = unsafe { CStr::from_ptr(self.inner.rc) };

        match str::from_utf8(cstr.to_bytes()) {
            Ok(s) => {
                if s.len() > 0 {
                    Some(s)
                }
                else {
                    None
                }
            },
            Err(_) => {
                None
            }
        }
    }
}

impl fmt::Debug for LibraryVersion {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let mut debug = fmt.debug_struct("LibraryVersion");

        debug.field("major", &self.major());
        debug.field("minor", &self.minor());
        debug.field("micro", &self.micro());
        debug.field("nano", &self.nano());
        debug.field("rc", &self.rc());

        debug.finish()
    }
}

/// Returns a structure with the version of the running libusb library.
pub fn version() -> LibraryVersion {
    let version: &'static ::libusb::libusb_version = unsafe {
        mem::transmute(::libusb::libusb_get_version())
    };

    LibraryVersion { inner: version }
}
