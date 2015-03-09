extern crate libusb;

fn main() {
  let mut context = libusb::Context::new().unwrap();
  let mut iface_handle = {
    let devices = context.devices().unwrap(); // ~ERROR: does not live long enough
    let mut dev = devices.iter().next().unwrap();
    let mut handle = dev.open().unwrap();
    handle.claim_interface(0).unwrap()
  };

  iface_handle.set_alternate_setting(0);
}
