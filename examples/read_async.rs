use rusb::{AsyncPool, Context, UsbContext};

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

    let mut buffers = Vec::new();
    for _ in 0..NUM_TRANSFERS {
        let buf = Vec::with_capacity(BUF_SIZE);
        buffers.push(buf);
    }

    let mut async_pool =
        AsyncPool::new_bulk(device, endpoint, buffers).expect("Failed to create async pool!");

    let mut swap_vec = Vec::with_capacity(BUF_SIZE);
    let timeout = Duration::from_secs(10);

    let mut num_bytes = 0u64;
    let mut num_transfers = 0u64;
    let mut last_time = std::time::Instant::now();
    loop {
        let poll_result = async_pool.poll(timeout, swap_vec);
        match poll_result {
            Ok(data) => {
                num_bytes += data.len() as u64;
                swap_vec = data
            }
            Err((err, buf)) => {
                eprintln!("Error: {}", err);
                swap_vec = buf
            }
        }
        num_transfers += 1;

        let elapsed = last_time.elapsed();
        if elapsed >= Duration::from_millis(1000) {
            let elapsed = elapsed.as_secs_f32();
            println!(
                "KiB per second: {}, \tTx per second: {}",
                num_bytes as f32 / 1024. / elapsed,
                num_transfers as f32 / elapsed
            );
            num_bytes = 0;
            num_transfers = 0;
            last_time = std::time::Instant::now();
        }
    }
}
