use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};

use rusb::{ffi, DeviceHandle, UsbContext};

use crate::{error::Error, error::Result, Transfer};

use std::sync::atomic::{AtomicBool, Ordering};

/// Represents a pool of asynchronous transfers, that can be polled to completion
pub struct TransferPool<C: UsbContext> {
    device: Arc<DeviceHandle<C>>,
    pending: VecDeque<Transfer>,
}

impl<C: UsbContext> TransferPool<C> {
    pub fn new(device: Arc<DeviceHandle<C>>) -> Self {
        Self {
            device,
            pending: VecDeque::new(),
        }
    }

    pub fn submit_bulk(&mut self, endpoint: u8, buf: Vec<u8>) -> Result<()> {
        // Safety: If transfer is submitted, it is pushed onto `pending` where it will be
        // dropped before `device` is freed.
        unsafe {
            let mut transfer = Transfer::bulk(self.device.as_raw(), endpoint, buf);
            transfer.submit()?;
            self.pending.push_back(transfer);
            Ok(())
        }
    }

    pub fn submit_control(
        &mut self,
        request_type: u8,
        request: u8,
        value: u16,
        index: u16,
        data: &[u8],
    ) -> Result<()> {
        // Safety: If transfer is submitted, it is pushed onto `pending` where it will be
        // dropped before `device` is freed.
        unsafe {
            let mut transfer = Transfer::control(
                self.device.as_raw(),
                request_type,
                request,
                value,
                index,
                data,
            );
            transfer.submit()?;
            self.pending.push_back(transfer);
            Ok(())
        }
    }

    pub unsafe fn submit_control_raw(&mut self, buffer: Vec<u8>) -> Result<()> {
        // Safety: If transfer is submitted, it is pushed onto `pending` where it will be
        // dropped before `device` is freed.
        unsafe {
            let mut transfer = Transfer::control_raw(self.device.as_raw(), buffer);
            transfer.submit()?;
            self.pending.push_back(transfer);
            Ok(())
        }
    }

    pub fn submit_interrupt(&mut self, endpoint: u8, buf: Vec<u8>) -> Result<()> {
        // Safety: If transfer is submitted, it is pushed onto `pending` where it will be
        // dropped before `device` is freed.
        unsafe {
            let mut transfer = Transfer::interrupt(self.device.as_raw(), endpoint, buf);
            transfer.submit()?;
            self.pending.push_back(transfer);
            Ok(())
        }
    }

    pub fn submit_iso(&mut self, endpoint: u8, buf: Vec<u8>, iso_packets: i32) -> Result<()> {
        // Safety: If transfer is submitted, it is pushed onto `pending` where it will be
        // dropped before `device` is freed.
        unsafe {
            let mut transfer = Transfer::iso(self.device.as_raw(), endpoint, buf, iso_packets);
            transfer.submit()?;
            self.pending.push_back(transfer);
            Ok(())
        }
    }

    pub fn poll(&mut self, timeout: Duration) -> Result<Vec<u8>> {
        let next = self.pending.front().ok_or(Error::NoTransfersPending)?;
        if poll_completed(self.device.context(), timeout, next.completed_flag()) {
            let mut transfer = self.pending.pop_front().unwrap();
            let res = transfer.handle_completed();
            res
        } else {
            Err(Error::PollTimeout)
        }
    }

    pub fn cancel_all(&mut self) {
        // Cancel in reverse order to avoid a race condition in which one
        // transfer is cancelled but another submitted later makes its way onto
        // the bus.
        for transfer in self.pending.iter_mut().rev() {
            transfer.cancel();
        }
    }

    /// Returns the number of async transfers pending
    pub fn pending(&self) -> usize {
        self.pending.len()
    }
}

unsafe impl<C: UsbContext> Send for TransferPool<C> {}
unsafe impl<C: UsbContext> Sync for TransferPool<C> {}

impl<C: UsbContext> Drop for TransferPool<C> {
    fn drop(&mut self) {
        self.cancel_all();
        while self.pending() > 0 {
            self.poll(Duration::from_secs(1)).ok();
        }
    }
}

/// This is effectively libusb_handle_events_timeout_completed, but with
/// `completed` as `AtomicBool` instead of `c_int` so it is safe to access
/// without the events lock held. It also continues polling until completion,
/// timeout, or error, instead of potentially returning early.
///
/// This design is based on
/// https://libusb.sourceforge.io/api-1.0/libusb_mtasync.html#threadwait
///
/// Returns `true` when `completed` becomes true, `false` on timeout, and panics on
/// any other libusb error.
fn poll_completed(ctx: &impl UsbContext, timeout: Duration, completed: &AtomicBool) -> bool {
    use ffi::constants::LIBUSB_ERROR_TIMEOUT;

    let deadline = Instant::now() + timeout;

    let mut err = 0;
    while err == 0 && !completed.load(Ordering::SeqCst) && deadline > Instant::now() {
        let remaining = deadline.saturating_duration_since(Instant::now());
        let timeval = libc::timeval {
            tv_sec: remaining.as_secs().try_into().unwrap(),
            tv_usec: remaining.subsec_micros().try_into().unwrap(),
        };
        unsafe {
            if ffi::libusb_try_lock_events(ctx.as_raw()) == 0 {
                if !completed.load(Ordering::SeqCst)
                    && ffi::libusb_event_handling_ok(ctx.as_raw()) != 0
                {
                    err = ffi::libusb_handle_events_locked(ctx.as_raw(), &timeval as *const _);
                }
                ffi::libusb_unlock_events(ctx.as_raw());
            } else {
                ffi::libusb_lock_event_waiters(ctx.as_raw());
                if !completed.load(Ordering::SeqCst)
                    && ffi::libusb_event_handler_active(ctx.as_raw()) != 0
                {
                    ffi::libusb_wait_for_event(ctx.as_raw(), &timeval as *const _);
                }
                ffi::libusb_unlock_event_waiters(ctx.as_raw());
            }
        }
    }

    match err {
        0 => completed.load(Ordering::SeqCst),
        LIBUSB_ERROR_TIMEOUT => false,
        _ => panic!(
            "Error {} when polling transfers: {}",
            err,
            unsafe { std::ffi::CStr::from_ptr(ffi::libusb_strerror(err)) }.to_string_lossy()
        ),
    }
}
