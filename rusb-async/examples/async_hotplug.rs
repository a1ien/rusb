use rusb::Error;
use rusb_async::{Context, HotplugBuilder, HotplugEvent, Registration};

#[tokio::main]
async fn main() -> Result<(), Error> {
    if !rusb::has_hotplug() {
        eprint!("libusb hotplug api unsupported");
        return Ok(());
    }

    let mut context = Context::new()?;

    let mut registration: Registration<Context> = HotplugBuilder::new()
        .enumerate(true)
        .register(&mut context)?;

    while let Some(event) = registration.next_event().await {
        match event {
            HotplugEvent::Arrived(device) => {
                println!("device arrived {:?}", device);
            }
            HotplugEvent::Left(device) => {
                println!("device left {:?}", device);
            }
        }
    }

    Ok(())
}
