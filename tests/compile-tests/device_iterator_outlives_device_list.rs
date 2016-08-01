extern crate libusb;

fn main() {
    let mut context = libusb::Context::new().unwrap();

    let mut iter = {
        let devices = context.devices().unwrap();
        devices.iter() // ~ERROR: does not live long enough
    };

    let dev = iter.next().unwrap();
}
