use rusb::{Context, Device, HotplugBuilder};

struct HotPlugHandler;

impl rusb::Hotplug for HotPlugHandler {
    fn device_arrived(&mut self, device: Device) {
        println!("device arrived {:?}", device);
    }

    fn device_left(&mut self, device: Device) {
        println!("device left {:?}", device);
    }
}

impl Drop for HotPlugHandler {
    fn drop(&mut self) {
        println!("HotPlugHandler dropped");
    }
}

fn main() -> rusb::Result<()> {
    if rusb::has_hotplug() {
        let context = Context::new()?;

        let mut reg = Some(
            HotplugBuilder::new()
                .enumerate(true)
                .register(context.clone(), Box::new(HotPlugHandler {}))?,
        );

        loop {
            context.handle_events(None).unwrap();
            if let Some(reg) = reg.take() {
                context.unregister_callback(reg);
                break;
            }
        }
        Ok(())
    } else {
        eprint!("libusb hotplug api unsupported");
        Ok(())
    }
}
