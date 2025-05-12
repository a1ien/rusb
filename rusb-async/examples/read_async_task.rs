//! This example also uses tokio, but instead of integrating file descriptor handling into
//! the event loop it takes a generic approach of a background task that handles immediate
//! events and then sleeps. This could very well be a background thread instead.

use rusb::{Context, UsbContext};
use rusb_async::transfer::BulkTransfer;
use tokio::task::JoinSet;

use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

async fn handle_events(context: Context) {
    loop {
        context.handle_events(Some(Duration::ZERO)).unwrap();
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 4 {
        eprintln!("Usage: read_async <base-10/0xbase-16> <base-10/0xbase-16> <endpoint>");
        return;
    }

    let vid = u16::from_str_radix(args[1].trim_start_matches("0x"), 16).unwrap();
    let pid = u16::from_str_radix(args[2].trim_start_matches("0x"), 16).unwrap();
    let endpoint: u8 = FromStr::from_str(args[3].as_ref()).unwrap();

    let ctx = Context::new().expect("Could not initialize libusb");
    tokio::spawn(handle_events(ctx.clone()));

    let device = Arc::new(
        ctx.open_device_with_vid_pid(vid, pid)
            .expect("Could not find device"),
    );

    const NUM_TRANSFERS: usize = 32;
    const BUF_SIZE: usize = 64;

    let mut join_set = JoinSet::new();

    for transfer_id in 0..NUM_TRANSFERS {
        let device = device.clone();

        join_set.spawn(async move {
            let mut bulk_transfer =
                BulkTransfer::new(device, endpoint, Vec::with_capacity(BUF_SIZE))
                    .expect("Failed to submit transfer");

            loop {
                let data = (&mut bulk_transfer).await.expect("Transfer failed");
                println!(
                    "Transfer id {transfer_id} got data: {} {:?}",
                    data.len(),
                    data
                );

                bulk_transfer
                    .reuse(endpoint, data)
                    .expect("Reusing allocated transfer failed");
            }
        });
    }

    join_set.join_all().await;
}
