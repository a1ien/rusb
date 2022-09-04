use rusb_async::TransferPool;

use rusb::{Context, UsbContext};

use std::time::Duration;
use std::{sync::Arc, thread};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 5 {
        eprintln!("Usage: read_write_async <vendor-id> <product-id> <out-endpoint> <in-endpoint> (all numbers hex)");
        return;
    }

    let vid = u16::from_str_radix(args[1].as_ref(), 16).unwrap();
    let pid = u16::from_str_radix(args[2].as_ref(), 16).unwrap();
    let out_endpoint = u8::from_str_radix(args[3].as_ref(), 16).unwrap();
    let in_endpoint = u8::from_str_radix(args[4].as_ref(), 16).unwrap();

    let ctx = Context::new().expect("Could not initialize libusb");
    let device = Arc::new(
        ctx.open_device_with_vid_pid(vid, pid)
            .expect("Could not find device"),
    );

    thread::spawn({
        let device = device.clone();
        move || {
            let mut write_pool = TransferPool::new(device).expect("Failed to create async pool!");

            let mut i = 0u8;

            loop {
                let mut buf = if write_pool.pending() < 8 {
                    Vec::with_capacity(64)
                } else {
                    write_pool
                        .poll(Duration::from_secs(5))
                        .expect("Failed to poll OUT transfer")
                };

                buf.clear();
                buf.push(i);
                buf.resize(64, 0x2);

                write_pool
                    .submit_bulk(out_endpoint, buf)
                    .expect("Failed to submit OUT transfer");
                println!("Wrote {}", i);
                i = i.wrapping_add(1);
            }
        }
    });

    let mut read_pool = TransferPool::new(device).expect("Failed to create async pool!");

    while read_pool.pending() < 8 {
        read_pool
            .submit_bulk(in_endpoint, Vec::with_capacity(1024))
            .expect("Failed to submit IN transfer");
    }

    loop {
        let data = read_pool
            .poll(Duration::from_secs(10))
            .expect("Failed to poll IN transfer");
        println!("Got data: {} {:?}", data.len(), data[0]);
        read_pool
            .submit_bulk(in_endpoint, data)
            .expect("Failed to resubmit IN transfer");
    }
}
