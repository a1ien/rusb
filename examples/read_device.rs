#![feature(core,std_misc)]

extern crate libusb;

use std::slice;

use std::str::FromStr;
use std::time::duration::Duration;

#[derive(Debug)]
struct Endpoint {
  config: u8,
  iface: u8,
  setting: u8,
  address: u8
}

fn main() {
  let args: Vec<String> = std::env::args().collect();

  if args.len() < 3 {
    println!("usage: show_device <vendor-id> <product-id>");
    return;
  }

  let vid: u16 = FromStr::from_str(args[1].as_slice()).unwrap();
  let pid: u16 = FromStr::from_str(args[2].as_slice()).unwrap();

  match libusb::Context::new() {
    Ok(mut context) => {
      match open_device(&mut context, vid, pid) {
        Some((device, mut handle)) => read_device(&device, &mut handle).unwrap(),
        None => println!("could not find device {:04x}:{:04x}", vid, pid)
      }
    },
    Err(e) => panic!("could not initialize libusb: {}", e)
  }
}

fn open_device(context: &mut libusb::Context, vid: u16, pid: u16) -> Option<(libusb::Device, libusb::DeviceHandle)> {
  let devices = match context.devices() {
    Ok(d) => d,
    Err(_) => return None
  };

  for mut device_ref in devices.iter() {
    let device = match device_ref.read_device() {
      Ok(d) => d,
      Err(_) => continue
    };

    if device.vendor_id() == vid && device.product_id() == pid {
      match device_ref.open() {
        Ok(handle) => return Some((device, handle)),
        Err(_) => continue
      }
    }
  }

  None
}

fn read_device(device: &libusb::Device, handle: &mut libusb::DeviceHandle) -> libusb::UsbResult<()> {
  try!(handle.reset());
  println!("Active configuration: {}", try!(handle.active_configuration()));

  match find_readable_endpoint(&device, libusb::TransferType::Interrupt) {
    Some(endpoint) => read_endpoint(handle, endpoint, libusb::TransferType::Interrupt),
    None => println!("No readable interrupt endpoint")
  }

  match find_readable_endpoint(&device, libusb::TransferType::Bulk) {
    Some(endpoint) => read_endpoint(handle, endpoint, libusb::TransferType::Bulk),
    None => println!("No readable bulk endpoint")
  }

  Ok(())
}

fn find_readable_endpoint(device: &libusb::Device, transfer_type: libusb::TransferType) -> Option<Endpoint> {
  for config in device.configurations() {
    for interface in config.interfaces() {
      for setting in interface.settings() {
        for endpoint in setting.endpoints() {
          if endpoint.direction() == libusb::Direction::In && endpoint.transfer_type() == transfer_type {
            return Some(Endpoint {
              config: config.number(),
              iface: interface.number(),
              setting: setting.number(),
              address: endpoint.address()
            });
          }
        }
      }
    }
  }

  None
}

fn read_endpoint(handle: &mut libusb::DeviceHandle, endpoint: Endpoint, transfer_type: libusb::TransferType) {
  println!("Reading from endpoint: {:?}", endpoint);

  let has_kernel_driver = match handle.kernel_driver_active(endpoint.iface) {
    Ok(true) => {
      handle.detach_kernel_driver(endpoint.iface).ok();
      true
    },
    _ => false
  };

  println!(" - kernel driver? {}", has_kernel_driver);

  match configure_endpoint(handle, &endpoint) {
    Ok(mut iface) => {
      let mut vec = Vec::<u8>::with_capacity(256);
      let mut buf = unsafe { slice::from_raw_parts_mut((&mut vec[..]).as_mut_ptr(), vec.capacity()) };

      let timeout = Duration::milliseconds(1000);

      match transfer_type {
        libusb::TransferType::Interrupt => {
          match iface.interrupt_transfer(endpoint.address, buf, timeout) {
            Ok(len) => {
              unsafe { vec.set_len(len) };
              println!(" - read: {:?}", vec);
            },
            Err(err) => println!("could not read from endpoint: {}", err)
          }
        },
        libusb::TransferType::Bulk => {
          match iface.bulk_transfer(endpoint.address, buf, timeout) {
            Ok(len) => {
              unsafe { vec.set_len(len) };
              println!(" - read: {:?}", vec);
            },
            Err(err) => println!("could not read from endpoint: {}", err)
          }
        },
        _ => ()
      }
    },
    Err(err) => println!("could not configure endpoint: {}", err)
  }

  if has_kernel_driver {
    handle.attach_kernel_driver(endpoint.iface).ok();
  }
}

fn configure_endpoint<'a>(handle: &'a mut libusb::DeviceHandle, endpoint: &Endpoint) -> libusb::UsbResult<libusb::InterfaceHandle<'a>> {
  try!(handle.set_active_configuration(endpoint.config));

  let mut iface = try!(handle.claim_interface(endpoint.iface));

  try!(iface.set_alternate_setting(endpoint.setting));

  Ok(iface)
}
