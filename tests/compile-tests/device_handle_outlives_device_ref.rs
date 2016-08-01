extern crate libusb;

fn main() {
    let mut context = libusb::Context::new().unwrap();

    let mut handle = {
        let devices = context.devices().unwrap();
        let mut dev = devices.iter().next().unwrap();
        dev.open().unwrap()
    };

    handle.active_configuration();
}
