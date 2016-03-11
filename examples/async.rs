extern crate libusb;
use std::str::FromStr;
use std::time::Duration;

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
        println!("usage: async <vendor-id> <product-id>");
        return;
    }

    let vid: u16 = FromStr::from_str(args[1].as_ref()).unwrap();
    let pid: u16 = FromStr::from_str(args[2].as_ref()).unwrap();

    let context = libusb::Context::new().unwrap();

    let (device, device_desc, mut handle) = open_device(&context, vid, pid).expect("Could not open device");
    read_device(&context, &device, &device_desc, &mut handle).unwrap();
}

fn open_device(context: &libusb::Context, vid: u16, pid: u16) -> Option<(libusb::Device, libusb::DeviceDescriptor, libusb::DeviceHandle)> {
    let devices = match context.devices() {
        Ok(d) => d,
        Err(_) => return None
    };

    for device in devices.iter() {
        let device_desc = match device.device_descriptor() {
            Ok(d) => d,
            Err(_) => continue
        };

        if device_desc.vendor_id() == vid && device_desc.product_id() == pid {
            match device.open() {
                Ok(handle) => return Some((device, device_desc, handle)),
                Err(_) => continue
            }
        }
    }

    None
}

fn read_device(context: &libusb::Context, device: &libusb::Device, device_desc: &libusb::DeviceDescriptor, handle: &mut libusb::DeviceHandle) -> libusb::Result<()> {
    match find_readable_endpoint(device, device_desc, libusb::TransferType::Interrupt) {
        Some(endpoint) => read_endpoint(context, handle, endpoint, libusb::TransferType::Interrupt),
        None => println!("No readable interrupt endpoint")
    }

    match find_readable_endpoint(device, device_desc, libusb::TransferType::Bulk) {
        Some(endpoint) => read_endpoint(context, handle, endpoint, libusb::TransferType::Bulk),
        None => println!("No readable bulk endpoint")
    }

    Ok(())
}

fn find_readable_endpoint(device: &libusb::Device, device_desc: &libusb::DeviceDescriptor, transfer_type: libusb::TransferType) -> Option<Endpoint> {
    for n in 0..device_desc.num_configurations() {
        let config_desc = match device.config_descriptor(n) {
            Ok(c) => c,
            Err(_) => continue
        };

        for interface in config_desc.interfaces() {
            for interface_desc in interface.descriptors() {
                for endpoint_desc in interface_desc.endpoint_descriptors() {
                    if endpoint_desc.direction() == libusb::Direction::In && endpoint_desc.transfer_type() == transfer_type {
                        return Some(Endpoint {
                            config: config_desc.number(),
                            iface: interface_desc.interface_number(),
                            setting: interface_desc.setting_number(),
                            address: endpoint_desc.address()
                        });
                    }
                }
            }
        }
    }

    None
}

fn read_endpoint(context: &libusb::Context, handle: &mut libusb::DeviceHandle, endpoint: Endpoint, transfer_type: libusb::TransferType) {
    println!("Reading from endpoint: {:?}", endpoint);

    configure_endpoint(handle, &endpoint).unwrap();

    let mut buffers = [[0u8; 128]; 8];

    {
        let mut async_group = ::libusb::AsyncGroup::new(context);
        let timeout = Duration::from_secs(1);

        match transfer_type {
            libusb::TransferType::Interrupt => {
                for buf in &mut buffers {
                    async_group.submit(::libusb::Transfer::interrupt(handle, endpoint.address, buf, timeout)).unwrap();
                }
            },
            libusb::TransferType::Bulk => {
                for buf in &mut buffers {
                    async_group.submit(::libusb::Transfer::bulk(handle, endpoint.address, buf, timeout)).unwrap();
                }
            }
            _ => unimplemented!()
        }

        loop {
            let mut transfer = async_group.wait_any().unwrap();
            println!("Read: {:?} {:?}", transfer.status(), transfer.actual());
            async_group.submit(transfer).unwrap();
        }
    }
}

fn configure_endpoint<'a>(handle: &'a mut libusb::DeviceHandle, endpoint: &Endpoint) -> libusb::Result<()> {
    handle.set_active_configuration(endpoint.config)?;
    handle.claim_interface(endpoint.iface)?;
    handle.set_alternate_setting(endpoint.iface, endpoint.setting)?;
    Ok(())
}
