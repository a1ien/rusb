extern crate libusb;

fn main() {
    let mut context = libusb::Context::new().unwrap();

    // no error, because devices have internal reference counting independent of device lists
    let mut dev = {
        let devices = context.devices().unwrap();
        devices.iter().next().unwrap()
    };

    let handle = dev.open().unwrap();
}
