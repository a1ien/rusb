use rusb::{Context, Device, UsbContext};
use std::option::Option::Some;

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
        let mut reg = Some(context.hotplug_register_callback(
            None,
            None,
            None,
            true,
            Box::new(HotPlugHandler {}),
        )?);
        loop {
            context.handle_events(None).unwrap();
            if let Some(reg) = reg.take() {
                context.hotplug_unregister_callback(reg);
            }
        }
    } else {
        eprint!("libusb hotplug api unsupported");
        Ok(())
    }
}
