use rusb::UsbContext;

fn main() {
    let version = rusb::version();

    println!(
        "libusb v{}.{}.{}.{}{}",
        version.major(),
        version.minor(),
        version.micro(),
        version.nano(),
        version.rc().unwrap_or("")
    );

    let mut context = match rusb::Context::new() {
        Ok(c) => c,
        Err(e) => panic!("libusb::Context::new(): {}", e),
    };

    context.set_log_level(rusb::LogLevel::Debug);
    context.set_log_level(rusb::LogLevel::Info);
    context.set_log_level(rusb::LogLevel::Warning);
    context.set_log_level(rusb::LogLevel::Error);
    context.set_log_level(rusb::LogLevel::None);

    println!("has capability? {}", rusb::has_capability());
    println!("has hotplug? {}", rusb::has_hotplug());
    println!("has HID access? {}", rusb::has_hid_access());
    println!(
        "supports detach kernel driver? {}",
        rusb::supports_detach_kernel_driver()
    )
}
