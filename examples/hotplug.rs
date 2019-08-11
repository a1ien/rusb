use rusb::{Context, Device, UsbContext};

struct HotPlugHandler;

impl<T: UsbContext> rusb::Hotplug<T> for HotPlugHandler {
    fn device_arrived(&mut self, device: Device<T>) {
        println!("device arrived {:?}", device);
    }

    fn device_left(&mut self, device: Device<T>) {
        println!("device left {:?}", device);
    }
}

fn main() -> rusb::Result<()> {
    if rusb::has_hotplug() {
        let context = Context::new()?;
        context.register_callback(None, None, None, Box::new(HotPlugHandler {}))?;

        loop {
            context.handle_events(None).unwrap();
        }
    } else {
        eprint!("libusb hotplug api unsupported");
        Ok(())
    }
}
