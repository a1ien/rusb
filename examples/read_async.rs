use rusb::{AsyncTransfer, CbResult, Context, UsbContext};

use std::str::FromStr;
use std::time::Duration;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 4 {
        eprintln!("Usage: read_async <vendor-id> <product-id> <endpoint>");
        return;
    }

    let vid: u16 = FromStr::from_str(args[1].as_ref()).unwrap();
    let pid: u16 = FromStr::from_str(args[2].as_ref()).unwrap();
    let endpoint: u8 = FromStr::from_str(args[3].as_ref()).unwrap();

    let ctx = Context::new().expect("Could not initialize libusb");
    let device = ctx
        .open_device_with_vid_pid(vid, pid)
        .expect("Could not find device");

    const NUM_TRANSFERS: usize = 32;
    const BUF_SIZE: usize = 1024;
    let mut main_buffer = Box::new([0u8; BUF_SIZE * NUM_TRANSFERS]);

    let mut transfers = Vec::new();
    for buf in main_buffer.chunks_exact_mut(BUF_SIZE) {
        let mut transfer =
            AsyncTransfer::new_bulk(&device, endpoint, buf, callback, Duration::from_secs(10));
        transfer.submit().expect("Could not submit transfer");
        transfers.push(transfer);
    }

    loop {
        rusb::poll_transfers(&ctx, Duration::from_secs(10));
    }
}

fn callback(result: CbResult) {
    println!("{:?}", result)
}
