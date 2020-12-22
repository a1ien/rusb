use std::fs;
use std::path::{Path, PathBuf};

fn get_api_version(libusb_source: &Path) {
    use std::io::BufRead;
    if let Ok(f) = fs::File::open(libusb_source) {
        let f = std::io::BufReader::new(f);
        for line in f.lines() {
            if let Ok(line) = line {
                if line.starts_with("#define LIBUSB_API_VERSION") {
                    if let Some(api_version) = line.rsplit(' ').next().and_then(|s| {
                        if s.starts_with("0x") {
                            let s = &s[2..];
                            u32::from_str_radix(s, 16).ok()
                        } else {
                            None
                        }
                    }) {
                        if api_version >= 0x01000108 {
                            println!("cargo:rustc-cfg=libusb_hotplug_get_user_data");
                        }
                    }
                    break;
                }
            }
        }
    }
}

fn main() {
    if let Ok(include_path) = std::env::var("DEP_USB_1.0_INCLUDE") {
        let path = PathBuf::from(include_path);
        get_api_version(path.join("libusb.h").as_path());
    }
}
