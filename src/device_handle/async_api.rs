use crate::{DeviceHandle, UsbContext};
use libusb1_sys as ffi;

use libc::c_void;
use std::collections::VecDeque;
use std::convert::TryInto;
use std::marker::PhantomPinned;
use std::pin::Pin;
use std::ptr::NonNull;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use thiserror::Error;

type CompletedQueue = Arc<Mutex<VecDeque<usize>>>;
type PinnedTransfer<C> = Pin<Box<AsyncTransfer<C>>>;

#[derive(Error, Debug)]
pub enum AsyncError {
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
    ) -> Pin<Box<Self>> {
        // non-isochronous endpoints (e.g. control, bulk, interrupt) specify a value of 0
        // This is step 1 of async API
        let ptr = unsafe { ffi::libusb_alloc_transfer(0) };
        let ptr = NonNull::new(ptr).expect("Could not allocate transfer!");

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
                0,
            )
        };
        result
    }

    // Part of step 4 of async API the transfer is finished being handled when
    // `poll()` is called.
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

    /// Prerequisite: self.buffer ans self.ptr are both correctly set
    fn swap_buffer(self: &mut AsyncTransfer<C>, new_buf: Vec<u8>) -> Vec<u8> {
        let transfer_struct = unsafe { self.ptr.as_mut() };

        let data = std::mem::replace(&mut self.buffer, new_buf);

        // Update transfer struct for new buffer
        transfer_struct.actual_length = 0; // TODO: Is this necessary?
        transfer_struct.buffer = self.buffer.as_mut_ptr();
        transfer_struct.length = self.buffer.capacity() as i32;

        data
    }

    // Step 3 of async API
    fn submit(self: &mut AsyncTransfer<C>) -> Result<(), AsyncError> {
        let transfer_struct = self.ptr;
        let errno = unsafe { ffi::libusb_submit_transfer(transfer_struct.as_ptr()) };

        use ffi::constants::*;
        use AsyncError as E;
        match errno {
            0 => Ok(()),
            LIBUSB_ERROR_NO_DEVICE => Err(E::Disconnected),
            LIBUSB_ERROR_BUSY => {
                unreachable!("We shouldn't be calling submit on transfers already submitted!")
            }
            LIBUSB_ERROR_NOT_SUPPORTED => Err(E::Other("Transfer not supported")),
            LIBUSB_ERROR_INVALID_PARAM => Err(E::Other("Transfer size bigger than OS supports")),
            _ => Err(E::Errno("Error while submitting transfer: ", errno)),
        }
    }
}
impl<C: UsbContext> Drop for AsyncTransfer<C> {
    fn drop(&mut self) {
        // TODO: Figure out how to destroy transfers, which is step 5 of async API.
        todo!()
    }
}

/// Represents a pool of asynchronous transfers, that can be polled to completion
pub struct AsyncPool<C: UsbContext> {
    /// Contains the pool of AsyncTransfers
    pool: Vec<PinnedTransfer<C>>,
    /// Contains the idxs of transfers in `pool` that have completed
    completed: CompletedQueue,
    device: Arc<DeviceHandle<C>>, // TODO: We hold refs to this, do we need it to be pinned?
}
impl<C: UsbContext> AsyncPool<C> {
    pub fn new_bulk(
        device: DeviceHandle<C>,
        endpoint: u8,
        buffers: impl IntoIterator<Item = Vec<u8>>,
    ) -> Result<Self, AsyncError> {
        let buffers = buffers.into_iter();
        let mut pool = Vec::with_capacity(buffers.size_hint().0);
        let completed = Arc::new(Mutex::new(VecDeque::with_capacity(buffers.size_hint().0)));
        let device = Arc::new(device);

        for (id, buf) in buffers.into_iter().enumerate() {
            let mut transfer: PinnedTransfer<C> =
                AsyncTransfer::new_bulk(id, completed.clone(), device.clone(), endpoint, buf);
            unsafe { transfer.as_mut().get_unchecked_mut() }.submit()?;
            pool.push(transfer);
        }
        Ok(Self {
            pool,
            completed,
            device,
        })
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

    /// Returns the number of async transfers in the pool.
    pub fn size(&self) -> usize {
        self.pool.len()
    }

    /// Once a transfer is completed, check the c struct for errors, otherwise swap
    /// buffers. Step 4 of async API.
    fn handle_completed_transfer(
        transfer: &mut PinnedTransfer<C>,
        new_buf: Vec<u8>,
    ) -> Result<Vec<u8>, (AsyncError, Vec<u8>)> {
        use AsyncError as E;
        let transfer = unsafe { transfer.as_mut().get_unchecked_mut() };
        let transfer_struct = unsafe { transfer.ptr.as_mut() };

        use ffi::constants::*;
        let result = match transfer_struct.status {
            LIBUSB_TRANSFER_COMPLETED => Ok(()),
            LIBUSB_TRANSFER_CANCELLED => Err(E::Cancelled),
            LIBUSB_TRANSFER_ERROR => Err(E::Other("Error occurred during transfer execution")),
            LIBUSB_TRANSFER_TIMED_OUT => {
                unreachable!("We are using timeout=0 which means no timeout")
            }
            LIBUSB_TRANSFER_STALL => Err(E::Stall),
            LIBUSB_TRANSFER_NO_DEVICE => Err(E::Disconnected),
            LIBUSB_TRANSFER_OVERFLOW => {
                panic!("Device sent more data than expected. Is this even possible when reading?")
            }
            _ => panic!("Found an unexpected error value for transfer status"),
        };
        match result {
            Err(err) => Err((err, new_buf)),
            Ok(()) => {
                debug_assert!(transfer_struct.length >= transfer_struct.actual_length); // sanity
                unsafe {
                    transfer
                        .buffer
                        .set_len(transfer_struct.actual_length as usize)
                };

                let data = transfer.swap_buffer(new_buf);

                let submit_result = transfer.submit();
                match submit_result {
                    Ok(()) => Ok(data),
                    Err(err) => {
                        // Take back the original buffer and return the error
                        let new_buf = transfer.swap_buffer(data);
                        Err((err, new_buf))
                    }
                }
            }
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
