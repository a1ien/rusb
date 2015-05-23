# Libusb
This crate provides a safe wrapper around the native `libusb` library. It applies the RAII pattern
and Rust lifetimes to ensure safe usage of all `libusb` functionality. The RAII pattern ensures that
all acquired resources are released when they're no longer needed, and Rust lifetimes ensure that
resources are released in a proper order.

## Dependencies
In order to use the `libusb` crate, you must have the native `libusb` library installed where it can
be found by `pkg-config`.

All systems supported by the native `libusb` library are also supported by the `libusb` crate. It's
been tested on Linux, OS X, and Windows.

### Cross-Compiling
The `libusb` crate can be used when cross-compiling to a foreign target. Details on how to
cross-compile `libusb` are explained in the [`libusb-sys` crate's
README](https://github.com/dcuddeback/libusb-sys#cross-compiling).

## Usage
Add `libusb` as a dependency in `Cargo.toml`:

```toml
[dependencies]
libusb = "0.1.0"
```

Import the `libusb` crate. The starting point for nearly all `libusb` functionality is to create a
context object. With a context object, you can list devices, read their descriptors, open them, and
communicate with their endpoints:

```rust
extern crate libusb;

fn main() {
  let mut context = libusb::Context::new().unwrap();

  for mut device_ref in context.devices().unwrap().iter() {
    let device = device_ref.read_device().unwrap();

    println!("Bus {:03} Device {:03} ID {:04x}:{:04x}",
        device.bus_number(), device.address(),
        device.vendor_id(), device.product_id());
  }
}
```

## Contributors
* [dcuddeback](https://github.com/dcuddeback)
* [nibua-r](https://github.com/nibua-r)

## License
Copyright © 2015 David Cuddeback

Distributed under the [MIT License](LICENSE).
