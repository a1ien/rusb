use std::env;
use std::io::Read;
use std::path::{Path, PathBuf};
use tar::Archive;

static VERSION: &'static str = "1.0.22";

fn link(name: &str, bundled: bool) {
    use std::env::var;
    let target = var("TARGET").unwrap();
    let target: Vec<_> = target.split('-').collect();
    if target.get(2) == Some(&"windows") {
        println!("cargo:rustc-link-lib=dylib={}", name);
        if bundled && target.get(3) == Some(&"gnu") {
            let dir = var("CARGO_MANIFEST_DIR").unwrap();
            println!("cargo:rustc-link-search=native={}/{}", dir, target[0]);
        }
    }
}

pub fn link_framework(name: &str) {
    println!("cargo:rustc-link-lib=framework={}", name);
}

#[cfg(target_env = "msvc")]
fn find_libusb_pkg() -> bool {
    match vcpkg::probe_package("libusb-1.0") {
        Ok(lib) => {
            for lib_dir in &lib.link_paths {
                println!("cargo:lib={}", lib_dir.to_str().unwrap());
            }
            for include_dir in &lib.include_paths {
                println!("cargo:include={}", include_dir.to_str().unwrap());
            }
            true
        }
        Err(e) => {
            println!("Can't find libusb pkg: {:?}", e);
            false
        }
    }
}

#[cfg(not(target_env = "msvc"))]
fn find_libusb_pkg() -> bool {
    match pkg_config::probe_library("libusb-1.0") {
        Ok(lib) => {
            for lib_dir in &lib.link_paths {
                println!("cargo:lib={}", lib_dir.to_str().unwrap());
            }
            for include_dir in &lib.include_paths {
                println!("cargo:include={}", include_dir.to_str().unwrap());
            }
            true
        }
        Err(e) => {
            println!("Can't find libusb pkg: {:?}", e);
            false
        }
    }
}

fn unpack<R: Read>(data: R, dst: &Path) -> std::io::Result<()> {
    let mut archive = Archive::new(data);
    let skip: PathBuf = "README".into();
    for entry in archive.entries()? {
        let mut entry = entry?;
        if entry.path()?.file_name().unwrap() == skip {
            continue;
        }
        entry.unpack_in(dst)?;
    }
    Ok(())
}

fn extract_source() -> PathBuf {
    use libflate::gzip::Decoder;
    use std::{fs, io::Cursor};

    let basename = format!("libusb-{}", VERSION);
    let filename = format!("libusb/{}.tar.gz", basename);

    let mut source_dir = PathBuf::from(env::var("OUT_DIR").unwrap()).join("source");
    let data = Cursor::new(fs::read(&filename).unwrap());
    let gz_decoder = Decoder::new(data).unwrap();
    unpack(gz_decoder, &source_dir).unwrap();
    source_dir.push(basename);
    source_dir
}

fn make_source() {
    let libusb_source = extract_source();

    std::fs::File::create(format!("{}/{}", libusb_source.display(), "config.h")).unwrap();
    let mut base_config = cc::Build::new();
    base_config.include(&libusb_source);
    base_config.include(libusb_source.join("libusb"));

    if cfg!(target_os = "macos") {
        base_config.define("OS_DARWIN", Some("1"));
        base_config.file(libusb_source.join("libusb/os/darwin_usb.c"));
        link_framework("CoreFoundation");
        link_framework("IOKit");
        link("objc", false);
    }
    if cfg!(target_os = "linux") {
        base_config.define("OS_LINUX", Some("1"));
        base_config.define("HAVE_ASM_TYPES_H", Some("1"));
        base_config.define("HAVE_LINUX_NETLINK_H", Some("1"));
        base_config.define("HAVE_SYS_SOCKET_H", Some("1"));
        base_config.define("USBI_TIMERFD_AVAILABLE", Some("1"));
        base_config.file(libusb_source.join("libusb/os/linux_netlink.c"));
        base_config.file(libusb_source.join("libusb/os/linux_usbfs.c"));
        base_config.define("POLL_NFDS_TYPE", Some("nfds_t"));
        base_config.define("_GNU_SOURCE", Some("1"));
    }

    if cfg!(unix) {
        base_config.define("HAVE_DLFCN_H", Some("1"));
        base_config.define("HAVE_GETTIMEOFDAY", Some("1"));
        base_config.define("HAVE_INTTYPES_H", Some("1"));
        base_config.define("HAVE_MEMORY_H", Some("1"));
        base_config.define("HAVE_POLL_H", Some("1"));
        base_config.define("HAVE_STDINT_H", Some("1"));
        base_config.define("HAVE_STDLIB_H", Some("1"));
        base_config.define("HAVE_STRINGS_H", Some("1"));
        base_config.define("HAVE_STRING_H", Some("1"));
        base_config.define("HAVE_STRUCT_TIMESPEC", Some("1"));
        base_config.define("HAVE_SYS_STAT_H", Some("1"));
        base_config.define("HAVE_SYS_TIME_H", Some("1"));
        base_config.define("HAVE_SYS_TYPES_H", Some("1"));
        base_config.define("HAVE_UNISTD_H", Some("1"));
        base_config.define("POLL_NFDS_TYPE", Some("nfds_t"));
        base_config.define("STDC_HEADERS", Some("1"));
        base_config.define("THREADS_POSIX", Some("1"));
        base_config.define(
            "DEFAULT_VISIBILITY",
            Some("__attribute__((visibility(\"default\")))"),
        );

        base_config.file(libusb_source.join("libusb/os/poll_posix.c"));
        base_config.file(libusb_source.join("libusb/os/threads_posix.c"));
    }

    if cfg!(windows) {
        #[cfg(target_env = "msvc")]
        base_config.define("_TIMESPEC_DEFINED", Some("1"));

        base_config.warnings(false);
        base_config.define("OS_WINDOWS", Some("1"));
        base_config.file(libusb_source.join("libusb/os/poll_windows.c"));
        base_config.file(libusb_source.join("libusb/os/threads_windows.c"));
        base_config.file(libusb_source.join("libusb/os/windows_nt_common.c"));
        base_config.file(libusb_source.join("libusb/os/windows_usbdk.c"));
        base_config.file(libusb_source.join("libusb/os/windows_winusb.c"));

        base_config.define("DEFAULT_VISIBILITY", Some(""));
        base_config.define("POLL_NFDS_TYPE", Some("unsigned int"));
        base_config.define("HAVE_SIGNAL_H", Some("1"));
        base_config.define("HAVE_SYS_TYPES_H", Some("1"));
        link("user32", false);
    }

    base_config.file(libusb_source.join("libusb/core.c"));
    base_config.file(libusb_source.join("libusb/descriptor.c"));
    base_config.file(libusb_source.join("libusb/hotplug.c"));
    base_config.file(libusb_source.join("libusb/io.c"));
    base_config.file(libusb_source.join("libusb/strerror.c"));
    base_config.file(libusb_source.join("libusb/sync.c"));

    base_config.compile("libusb.a");
}

fn main() {
    if !find_libusb_pkg() {
        extract_source();
        make_source();
    }
}
