extern crate libusb;

fn main() {
  let devs = {
    let mut context = libusb::Context::new().unwrap();
    context.devices().unwrap() // ~ERROR: does not live long enough
  };

  for mut dev in devs.iter() {
    dev.open().unwrap();
  }
}
