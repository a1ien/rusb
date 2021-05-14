use crate::{DeviceHandle, UsbContext};
use libusb1_sys as ffi;

use libc::c_void;
use std::collections::VecDeque;
use std::convert::{TryFrom, TryInto};
use std::marker::PhantomPinned;
use std::pin::Pin;
use std::ptr::NonNull;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use thiserror::Error;

type CompletedQueue = Arc<Mutex<VecDeque<usize>>>;
type Transfer<C> = Pin<Box<AsyncTransfer<C>>>;

#[derive(Error, Debug)]
pub enum AsyncError {
    #[error("Transfer timed out")]
    TransferTimeout,
    #[error("Poll timed out")]
    PollTimeout,
    #[error("Transfer is stalled")]
    Stall,
    #[error("Device was disconnected")]
    Disconnected,
    #[error("Other Error: {0}")]
    Other(&'static str),
    #[error("{0}ERRNO: {1}")]
    Errno(&'static str, i32),
    #[error("Transfer was cancelled")]
    Cancelled,
}

struct AsyncTransfer<C: UsbContext> {
    ptr: NonNull<ffi::libusb_transfer>,
    /// The ID of the transfer, as an idx into the transfer pool
    pool_id: usize,
    buffer: Vec<u8>,
    completed_queue: CompletedQueue,
    device: Arc<DeviceHandle<C>>,
    // `ptr` holds a pointer to `buffer`, `device`, and `self`, so we must be `!Unpin`
    _pin: PhantomPinned,
}
impl<C: UsbContext> AsyncTransfer<C> {
    fn new_bulk(
        pool_id: usize,
        completed_queue: CompletedQueue,
        device: Arc<DeviceHandle<C>>,
        endpoint: u8,
        buffer: Vec<u8>,
        timeout: std::time::Duration,
    ) -> Pin<Box<Self>> {
        // non-isochronous endpoints (e.g. control, bulk, interrupt) specify a value of 0
        // This is step 1 of async API
        let ptr = unsafe { ffi::libusb_alloc_transfer(0) };
        let ptr = NonNull::new(ptr).expect("Could not allocate transfer!");
        let timeout = libc::c_uint::try_from(timeout.as_millis())
            .expect("Duration was too long to fit into a c_uint");

        // Safety: Pinning `result` ensures it doesn't move, but we know that we will
        // want to access its fields mutably, we just don't want its memory location
        // changing (or its fields moving!). So routinely we will unsafely interact with
        // its fields mutably through a shared reference, but this is still sound.
        let result = Box::pin(Self {
            ptr,
            pool_id,
            buffer,
            completed_queue,
            device,
            _pin: PhantomPinned,
        });
        let user_data = std::ptr::addr_of!(*result) as *mut c_void;

        unsafe {
            // Step 2 of async api
            ffi::libusb_fill_bulk_transfer(
                ptr.as_ptr(),
                result.device.as_raw(),
                endpoint,
                result.buffer.as_ptr() as *mut u8,
                result.buffer.capacity().try_into().unwrap(),
                Self::transfer_cb,
                user_data, // We haven't associated with a queue yet, so this is null
                timeout,
            )
        };
        result
    }

    // We need to invoke our closure using a c-style function, so we store the closure
    // inside the custom user data field of the transfer struct, and then call the
    // user provided closure from there.
    // Step 4 of async API
    extern "system" fn transfer_cb(transfer: *mut ffi::libusb_transfer) {
        // Safety: libusb should never make this null, so this is fine
        let transfer = unsafe { &mut *transfer };

        // sanity
        debug_assert_eq!(
            transfer.transfer_type,
            ffi::constants::LIBUSB_TRANSFER_TYPE_BULK
        );

        let transfer: *mut AsyncTransfer<C> = transfer.user_data.cast();
        let transfer = unsafe { &mut *transfer };
        // Mark transfer as completed
        transfer
            .completed_queue
            .lock()
            .unwrap()
            .push_back(transfer.pool_id);
    }

    fn submit(self: &mut Transfer<C>) -> Result<(), AsyncError> {
        todo!()
    }
}
// TODO: Figure out how to destroy transfers

/// Represents a pool of asynchronous transfers, that can be polled to completion
pub struct AsyncPool<C: UsbContext> {
    /// Contains the pool of AsyncTransfers
    pool: Vec<Transfer<C>>,
    /// Contains the idxs of transfers in the `pool` that have completed
    completed: CompletedQueue,
    device: Arc<DeviceHandle<C>>, // TODO: We hold refs to this, do we need it to be pinned?
}
impl<C: UsbContext> AsyncPool<C> {
    pub fn new_bulk(
        device: DeviceHandle<C>,
        endpoint: u8,
        read_timeout: Duration,
        buffers: impl IntoIterator<Item = Vec<u8>>,
    ) -> Result<Self, AsyncError> {
        let buffers = buffers.into_iter();
        let mut pool = Vec::with_capacity(buffers.size_hint().0);
        let completed = Arc::new(Mutex::new(VecDeque::with_capacity(buffers.size_hint().0)));
        let device = Arc::new(device);

        for (id, buf) in buffers.into_iter().enumerate() {
            let mut transfer: Transfer<C> = AsyncTransfer::new_bulk(
                id,
                completed.clone(),
                device.clone(),
                endpoint,
                buf,
                read_timeout,
            );
            transfer.submit()?;
            pool.push(transfer);
        }
        Ok(Self {
            pool,
            completed,
            device,
        })
    }

    /// Once a transfer is completed, check the c struct for errors, otherwise swap
    /// buffers
    fn handle_completed_transfer(
        transfer: &mut Transfer<C>,
        new_buf: Vec<u8>,
    ) -> Result<Vec<u8>, (AsyncError, Vec<u8>)> {
        todo!()
    }

    /// Polls for the completion of async transfers. If successful, will swap the buffer
    /// of the completed transfer with `new_buf`, otherwise returns `(err, new_buf)` so
    /// that `new_buf` may be repurposed.
    pub fn poll(
        &mut self,
        timeout: Duration,
        new_buf: Vec<u8>,
    ) -> Result<Vec<u8>, (AsyncError, Vec<u8>)> {
        let pop_result = { self.completed.lock().unwrap().pop_front() };
        if let Some(id) = pop_result {
            let ref mut transfer = self.pool[id];
            return Self::handle_completed_transfer(transfer, new_buf);
        }
        // No completed transfers, so poll for some new ones
        poll_transfers(self.device.context(), timeout);
        let pop_result = { self.completed.lock().unwrap().pop_front() };
        if let Some(id) = pop_result {
            let ref mut transfer = self.pool[id];
            Self::handle_completed_transfer(transfer, new_buf)
        } else {
            Err((AsyncError::PollTimeout, new_buf))
        }
    }
}

/// Polls for transfers and executes their callbacks. Will block until the
/// given timeout, or return immediately if timeout is zero.
fn poll_transfers(ctx: &impl UsbContext, timeout: Duration) {
    let timeval = libc::timeval {
        tv_sec: timeout.as_secs().try_into().unwrap(),
        tv_usec: timeout.subsec_millis().try_into().unwrap(),
    };
    unsafe {
        let errno = ffi::libusb_handle_events_timeout_completed(
            ctx.as_raw(),
            std::ptr::addr_of!(timeval),
            std::ptr::null_mut(),
        );
        use ffi::constants::*;
        match errno {
            0 => (),
            LIBUSB_ERROR_INVALID_PARAM => panic!("Provided timeout was unexpectedly invalid"),
            _ => panic!(
                "Error when polling transfers. ERRNO: {}, Message: {}",
                errno,
                std::ffi::CStr::from_ptr(ffi::libusb_strerror(errno)).to_string_lossy()
            ),
        }
    }
}
