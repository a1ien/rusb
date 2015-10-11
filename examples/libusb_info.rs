extern crate libusb;

fn main() {
    let version = libusb::version();

    println!("libusb v{}.{}.{}.{}{}", version.major(), version.minor(), version.micro(), version.nano(), version.rc().unwrap_or(""));

    let mut context = match libusb::Context::new() {
        Ok(c) => c,
        Err(e) => panic!("libusb::Context::new(): {}", e)
    };

    context.set_log_level(libusb::LogLevel::Debug);
    context.set_log_level(libusb::LogLevel::Info);
    context.set_log_level(libusb::LogLevel::Warning);
    context.set_log_level(libusb::LogLevel::Error);
    context.set_log_level(libusb::LogLevel::None);

    println!("has capability? {}", context.has_capability());
    println!("has hotplug? {}", context.has_hotplug());
    println!("has HID access? {}", context.has_hid_access());
    println!("supports detach kernel driver? {}", context.supports_detach_kernel_driver())
}
