# Rusb
This crate provides a safe wrapper around the native `libusb` library. It applies the RAII pattern
and Rust lifetimes to ensure safe usage of all `libusb` functionality. The RAII pattern ensures that
all acquired resources are released when they're no longer needed, and Rust lifetimes ensure that
resources are released in a proper order.

* [Documentation](https://docs.rs/rusb)

## Dependencies
To use rusb no extra setup is required as rusb will automatically download the source for libusb and build it.

However if building libusb fails you can also try setting up the native `libusb` library where it can
be found by `pkg-config` or `vcpkg`.

All systems supported by the native `libusb` library are also supported by the `libusb` crate. It's
been tested on Linux, OS X, and Windows.

### Cross-Compiling
The `rusb` crate can be used when cross-compiling to a foreign target. Details on how to
cross-compile `rusb` are explained in the [`libusb1-sys` crate's
README](libusb1-sys/README.md#cross-compiling).

## Usage
Add `rusb` as a dependency in `Cargo.toml`:

```toml
[dependencies]
rusb = "0.8"
```

Import the `rusb` crate. The starting point for nearly all `rusb` functionality is to create a
context object. With a context object, you can list devices, read their descriptors, open them, and
communicate with their endpoints:

```rust
fn main() {
    for device in rusb::devices().unwrap().iter() {
        let device_desc = device.device_descriptor().unwrap();

        println!("Bus {:03} Device {:03} ID {:04x}:{:04x}",
            device.bus_number(),
            device.address(),
            device_desc.vendor_id(),
            device_desc.product_id());
    }
}
```

## Contributors
* [dcuddeback](https://github.com/dcuddeback)
* [nibua-r](https://github.com/nibua-r)
* [kevinmehall](https://github.com/kevinmehall)

## License
Copyright © 2015 David Cuddeback

Distributed under the [MIT License](LICENSE).
