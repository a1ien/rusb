use libc::{POLLIN, POLLOUT};
use rusb::{Context, UsbContext};
use rusb_async::fd::{FdCallbacks, FdEventHandler};
use rusb_async::transfer::BulkTransfer;
use tokio::io::unix::AsyncFd;
use tokio::io::Interest;
use tokio::task::{JoinHandle, JoinSet};

use std::collections::BTreeMap;
use std::os::fd::RawFd;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[cfg(not(unix))]
compile_error!("File descriptor event loop integrations are only supported on UNIX-like systems");

struct TokioFdCallbacks {
    context: Context,
    fd_handle_map: Mutex<BTreeMap<RawFd, JoinHandle<()>>>,
}

impl TokioFdCallbacks {
    pub fn new(context: Context) -> Self {
        Self {
            context,
            fd_handle_map: Mutex::new(BTreeMap::new()),
        }
    }
}

impl FdCallbacks for TokioFdCallbacks {
    type Context = Context;

    fn context(&self) -> &Self::Context {
        &self.context
    }

    fn fd_added(&self, fd: RawFd, events: libc::c_short) {
        let mut interest: Option<Interest> = None;

        if events & POLLIN != 0 {
            interest = Some(Interest::READABLE);
        }

        if events & POLLOUT != 0 {
            interest = interest
                .map(|i| i.add(Interest::WRITABLE))
                .or(Some(Interest::WRITABLE));
        }

        let Some(interest) = interest else {
            return;
        };

        let async_fd = AsyncFd::with_interest(fd, interest).unwrap();

        let context = self.context.clone();

        let handle = tokio::spawn(async move {
            loop {
                let mut guard = async_fd.ready(interest).await.unwrap();
                context.handle_events(Some(Duration::ZERO)).unwrap();
                guard.clear_ready();
            }
        });

        self.fd_handle_map.lock().unwrap().insert(fd, handle);
    }

    fn fd_removed(&self, fd: RawFd) {
        if let Some(handle) = self.fd_handle_map.lock().unwrap().remove(&fd) {
            handle.abort();
        }
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
    // Must not be dropped until all futures resolved, or else they will never complete.
    let _fd_event_handler = FdEventHandler::new(TokioFdCallbacks::new(ctx.clone()));

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
