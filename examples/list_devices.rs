extern crate libusb;

fn main() {
  list_devices().unwrap();
}

fn list_devices() -> libusb::UsbResult<()> {
  let mut context = try!(libusb::Context::new());

  for mut device_ref in try!(context.devices()).iter() {
    let device = match device_ref.read_device() {
      Ok(d) => d,
      Err(_) => continue
    };

    println!("Bus {:03} Device {:03} ID {:04x}:{:04x} {}", device.bus_number(), device.address(), device.vendor_id(), device.product_id(), get_speed(device.speed()));
    print_device(&device);

    for config in device.configurations() {
      print_config(&config);

      for interface in config.interfaces() {
        for setting in interface.settings() {
          print_interface(&setting, interface.number());

          for endpoint in setting.endpoints() {
            print_endpoint(&endpoint);
          }
        }
      }
    }
  }

  Ok(())
}

fn print_device(device: &libusb::Device) {
  println!("Device Descriptor:");
  println!("  bcdUSB             {:2}.{}{}", device.usb_version().major(), device.usb_version().minor(), device.usb_version().sub_minor());
  println!("  bDeviceClass        {:#04x}", device.class_code());
  println!("  bDeviceSubClass     {:#04x}", device.sub_class_code());
  println!("  bDeviceProtocol     {:#04x}", device.protocol_code());
  println!("  bMaxPacketSize0      {:3}", device.max_packet_size());
  println!("  idVendor          {:#06x}", device.vendor_id());
  println!("  idProduct         {:#06x}", device.product_id());
  println!("  bcdDevice          {:2}.{}{}", device.device_version().major(), device.device_version().minor(), device.device_version().sub_minor());
  println!("  bNumConfigurations   {:3}", device.configurations().len());
}

fn print_config(config: &libusb::Configuration) {
  println!("  Config Descriptor:");
  println!("    bNumInterfaces       {:3}", config.interfaces().len());
  println!("    bConfigurationValue  {:3}", config.number());
  println!("    bmAttributes:");
  println!("      Self Powered     {:>5}", config.self_powered());
  println!("      Remote Wakeup    {:>5}", config.remote_wakeup());
  println!("    bMaxPower           {:4}mW", config.max_power());
}

fn print_interface(setting: &libusb::InterfaceSetting, iface: u8) {
  println!("    Interface Descriptor:");
  println!("      bInterfaceNumber     {:3}", iface);
  println!("      bAlternateSetting    {:3}", setting.number());
  println!("      bNumEndpoints        {:3}", setting.endpoints().len());
  println!("      bInterfaceClass     {:#04x}", setting.class_code());
  println!("      bInterfaceSubClass  {:#04x}", setting.sub_class_code());
  println!("      bInterfaceProtocol  {:#04x}", setting.protocol_code());
}

fn print_endpoint(endpoint: &libusb::Endpoint) {
  println!("      Endpoint Descriptor:");
  println!("        bEndpointAddress    {:#04x} EP {} {:?}", endpoint.address(), endpoint.number(), endpoint.direction());
  println!("        bmAttributes:");
  println!("          Transfer Type          {:?}", endpoint.transfer_type());
  println!("          Synch Type             {:?}", endpoint.sync_type());
  println!("          Usage Type             {:?}", endpoint.usage_type());
  println!("        wMaxPacketSize    {:#06x}", endpoint.max_packet_size());
  println!("        bInterval            {:3}", endpoint.interval());
}

fn get_speed(speed: libusb::Speed) -> &'static str {
  match speed {
    libusb::Speed::Super   => "5000 Mbps",
    libusb::Speed::High    => " 480 Mbps",
    libusb::Speed::Full    => "  12 Mbps",
    libusb::Speed::Low     => " 1.5 Mbps",
    libusb::Speed::Unknown => "(unknown)"
  }
}
