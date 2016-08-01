extern crate libusb;

fn main() {
    let mut dev = {
        let mut context = libusb::Context::new().unwrap();
        let devices = context.devices().unwrap(); // ~ERROR: does not live long enough
        devices.iter().next().unwrap()
    };

    let handle = dev.open().unwrap();
}
