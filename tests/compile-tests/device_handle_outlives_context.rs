extern crate libusb;

fn main() {
  let mut handle = {
    let mut context = libusb::Context::new().unwrap();
    let devices = context.devices().unwrap(); // ~ERROR: does not live long enough
    let mut dev = devices.iter().next().unwrap();
    dev.open().unwrap()
  };

  handle.active_configuration();
}
