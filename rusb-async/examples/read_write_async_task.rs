//! This example also uses tokio, but instead of integrating file descriptor handling into
//! the event loop it takes a generic approach of a background task that handles immediate
//! events and then sleeps. This could very well be a background thread instead.

use rusb::{Context, UsbContext};
use rusb_async::transfer::BulkTransfer;
use tokio::task::JoinSet;

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

    if args.len() < 5 {
        eprintln!("Usage: read_write_async <vendor-id> <product-id> <out-endpoint> <in-endpoint> (all numbers hex)");
        return;
    }

    let vid = u16::from_str_radix(args[1].as_ref(), 16).unwrap();
    let pid = u16::from_str_radix(args[2].as_ref(), 16).unwrap();
    let out_endpoint = u8::from_str_radix(args[3].as_ref(), 16).unwrap();
    let in_endpoint = u8::from_str_radix(args[4].as_ref(), 16).unwrap();

    let ctx = Context::new().expect("Could not initialize libusb");
    tokio::spawn(handle_events(ctx.clone()));

    let device = Arc::new(
        ctx.open_device_with_vid_pid(vid, pid)
            .expect("Could not find device"),
    );

    const NUM_TRANSFERS: usize = 8;

    let mut join_set = JoinSet::new();

    for write_transfer_id in 0..NUM_TRANSFERS {
        let device = device.clone();

        join_set.spawn(async move {
            let i = 0;
            let mut bulk_transfer = BulkTransfer::new(device, out_endpoint, Vec::with_capacity(64))
                .expect("Failed to submit OUT transfer");

            loop {
                let data = (&mut bulk_transfer).await.expect("OUT Transfer failed");
                println!("OUT transfer {write_transfer_id} wrote {i}");
                bulk_transfer
                    .reuse(out_endpoint, data)
                    .expect("Reusing allocated OUT transfer failed");
            }
        });
    }

    for read_transfer_id in 0..NUM_TRANSFERS {
        let device = device.clone();

        join_set.spawn(async move {
            let mut bulk_transfer =
                BulkTransfer::new(device, in_endpoint, Vec::with_capacity(1024))
                    .expect("Failed to submit IN transfer");

            loop {
                let data = (&mut bulk_transfer).await.expect("IN Transfer failed");
                println!(
                    "IN transfer {read_transfer_id} got data: {} {:?}",
                    data.len(),
                    data
                );

                bulk_transfer
                    .reuse(in_endpoint, data)
                    .expect("Reusing allocated IN transfer failed");
            }
        });
    }

    join_set.join_all().await;
}
