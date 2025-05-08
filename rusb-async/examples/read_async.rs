use rusb::{Context, UsbContext};

use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

fn convert_argument(input: &str) -> u16 {
    if input.starts_with("0x") {
        return u16::from_str_radix(input.trim_start_matches("0x"), 16).unwrap();
    }
    u16::from_str_radix(input, 10)
        .expect("Invalid input, be sure to add `0x` for hexadecimal values.")
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 4 {
        eprintln!("Usage: read_async <base-10/0xbase-16> <base-10/0xbase-16> <endpoint>");
        return;
    }

    let vid = convert_argument(args[1].as_ref());
    let pid = convert_argument(args[2].as_ref());
    let endpoint: u8 = FromStr::from_str(args[3].as_ref()).unwrap();

    let ctx = Context::new().expect("Could not initialize libusb");
    let device = Arc::new(
        ctx.open_device_with_vid_pid(vid, pid)
            .expect("Could not find device"),
    );

    const NUM_TRANSFERS: usize = 32;
    const BUF_SIZE: usize = 64;

    let mut async_pool = TransferPool::new(device).expect("Failed to create async pool!");

    while async_pool.pending() < NUM_TRANSFERS {
        async_pool
            .submit_bulk(endpoint, Vec::with_capacity(BUF_SIZE))
            .expect("Failed to submit transfer");
    }

    let timeout = Duration::from_secs(10);
    loop {
        let data = async_pool.poll(timeout).expect("Transfer failed");
        println!("Got data: {} {:?}", data.len(), data);
        async_pool
            .submit_bulk(endpoint, data)
            .expect("Failed to resubmit transfer");
    }
}
