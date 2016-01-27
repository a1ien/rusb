# Libusb Rust Bindings

The `libusb-sys` crate provides declarations and linkage for the `libusb` C library. Following the
`*-sys` package conventions, the `libusb-sys` crate does not define higher-level abstractions over
the native `libusb` library functions.

## Dependencies
In order to use the `libusb-sys` crate, you must have the `libusb` library installed where it can be
found by `pkg-config`.

All systems supported by `libusb` are also supported by the `libusb-sys` crate. It's been tested on
Linux, OS X, and Windows.

### Cross-Compiling
To link to a cross-compiled version of the native `libusb` library, it's necessary to set several
environment variables to configure `pkg-config` to work with a cross-compiler's sysroot. [Autotools
Mythbuster](https://autotools.io/) has a good explanation of [supporting
cross-compilation](https://autotools.io/pkgconfig/cross-compiling.html) with `pkg-config`.

However, Rust's [`pkg-config` build helper](https://github.com/alexcrichton/pkg-config-rs) doesn't
support calling a `$CHOST`-prefixed `pkg-config`. It will always call `pkg-config` without a prefix.
To cross-compile `libusb-sys` with the `pkg-config` build helper, one must define the environment
variables `PKG_CONFIG_DIR`, `PKG_CONFIG_LIBDIR`, and `PKG_CONFIG_SYSROOT_DIR` for the *default*
`pkg-config`. It's also necessary to set `PKG_CONFIG_ALLOW_CROSS` to tell Rust's `pkg-config` helper
that it's okay to proceed with a cross-compile.

To adapt the `pkg-config` wrapper in the Autotools Mythbuster guide so that it works with Rust, one
will end up with a script similar to the following:

```sh
#!/bin/sh

SYSROOT=/build/root

export PKG_CONFIG_DIR=
export PKG_CONFIG_LIBDIR=${SYSROOT}/usr/lib/pkgconfig:${SYSROOT}/usr/share/pkgconfig
export PKG_CONFIG_SYSROOT_DIR=${SYSROOT}
export PKG_CONFIG_ALLOW_CROSS=1

cargo build
```

## Usage
Add `libusb-sys` as a dependency in `Cargo.toml`:

```toml
[dependencies]
libusb-sys = "0.2"
```

Import the `libusb_sys` crate and use the functions as they're defined in the native `libusb`
library. See the [`libusb` 1.0 API documention](http://libusb.sourceforge.net/api-1.0/) for more
usage information.

```rust
extern crate libusb_sys as ffi;

fn main() {
  let version = unsafe { ffi::libusb_get_version() };

  println!("libusb v{}.{}.{}.{}", version.major, version.minor, version.micro, version.nano);
}
```

### Finding Help
Since `libusb-sys` is no more than a wrapper around the native `libusb` library, the best source for
help is the information already available for `libusb`:

* [Home Page](http://libusb.info/)
* [API Documentation](http://libusb.sourceforge.net/api-1.0/)
* [Source](https://github.com/libusb/libusb)


## License
Copyright © 2015 David Cuddeback

Distributed under the [MIT License](LICENSE).
