use rusb::UsbContext as _;
use rusb_async::{Context, DeviceHandleExt as _};

use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

const BUF_SIZE: usize = 64;

fn convert_argument(input: &str) -> u16 {
    if input.starts_with("0x") {
        return u16::from_str_radix(input.trim_start_matches("0x"), 16).unwrap();
    }
    u16::from_str_radix(input, 10)
        .expect("Invalid input, be sure to add `0x` for hexadecimal values.")
}

#[tokio::main]
async fn main() {
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

    let timeout = Duration::from_secs(10);
    let mut buffer = vec![0u8; BUF_SIZE].into_boxed_slice();

    loop {
        let (bytes, n) = device
            .read_bulk_async(endpoint, buffer, timeout)
            .await
            .expect("Failed to submit transfer");

        println!("Got data: {} {:?}", n, &bytes[..n]);

        buffer = bytes;
    }
}
