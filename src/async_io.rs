use libc::{c_int, c_uchar, c_uint, c_void};
use libusb1_sys::*;
use std::cell::UnsafeCell;
use std::collections::{HashSet, VecDeque};
use std::{marker::PhantomData, mem, slice, sync::Mutex, time::Duration};

use crate::{constants::*, Context, DeviceHandle, Error, Result};

/// An asynchronous transfer that is not currently pending.
/// Specifies the data necessary to perform a transfer on a specified endpoint, and holds the
/// result of a completed transfer. A completed Transfer can be resubmitted.
pub struct Transfer<'d> {
    _handle: PhantomData<&'d DeviceHandle<'d>>, // transfer.dev_handle
    _buffer: PhantomData<&'d mut [u8]>,         // transfer.data
    transfer: *mut libusb1_sys::libusb_transfer,
}

/// The status of a Transfer returned by wait_any.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TransferStatus {
    /// Completed without error
    Success = LIBUSB_TRANSFER_COMPLETED as isize,

    /// Failed (IO error)
    Error = LIBUSB_TRANSFER_ERROR as isize,

    /// Timed out
    Timeout = LIBUSB_TRANSFER_TIMED_OUT as isize,

    /// Cancelled
    Cancelled = LIBUSB_TRANSFER_CANCELLED as isize,

    /// Endpoint stalled or control request not supported
    Stall = LIBUSB_TRANSFER_STALL as isize,

    /// Device was disconnected
    NoDevice = LIBUSB_TRANSFER_NO_DEVICE as isize,

    /// Device sent more data than requested
    Overflow = LIBUSB_TRANSFER_OVERFLOW as isize,

    /// No status, not yet submitted
    Unknown = -1 as isize,
}

impl<'d> Transfer<'d> {
    fn new(
        handle: &'d DeviceHandle<'d>,
        endpoint: u8,
        transfer_type: c_uchar,
        buffer: &'d mut [u8],
        timeout: Duration,
    ) -> Transfer<'d> {
        let timeout_ms = timeout.as_secs() * 1000 + u64::from(timeout.subsec_nanos()) / 1_000_000;
        unsafe {
            let t = libusb_alloc_transfer(0);
            (*t).status = -1;
            (*t).dev_handle = handle.as_raw();
            (*t).endpoint = endpoint as c_uchar;
            (*t).transfer_type = transfer_type;
            (*t).timeout = timeout_ms as c_uint;
            (*t).buffer = buffer.as_mut_ptr();
            (*t).length = buffer.len() as i32;
            (*t).actual_length = 0;

            Transfer {
                transfer: t,
                _handle: PhantomData,
                _buffer: PhantomData,
            }
        }
    }

    /// Creates an asynchronous bulk transfer, but does not submit it.
    pub fn bulk(
        handle: &'d DeviceHandle<'d>,
        endpoint: u8,
        buffer: &'d mut [u8],
        timeout: Duration,
    ) -> Transfer<'d> {
        Transfer::new(handle, endpoint, LIBUSB_TRANSFER_TYPE_BULK, buffer, timeout)
    }

    /// Creates an asynchronous interrupt transfer, but does not submit it.
    pub fn interrupt(
        handle: &'d DeviceHandle<'d>,
        endpoint: u8,
        buffer: &'d mut [u8],
        timeout: Duration,
    ) -> Transfer<'d> {
        Transfer::new(
            handle,
            endpoint,
            LIBUSB_TRANSFER_TYPE_INTERRUPT,
            buffer,
            timeout,
        )
    }

    /// Gets the status of a completed transfer.
    pub fn status(&self) -> TransferStatus {
        match unsafe { (*self.transfer).status } {
            LIBUSB_TRANSFER_COMPLETED => TransferStatus::Success,
            LIBUSB_TRANSFER_ERROR => TransferStatus::Error,
            LIBUSB_TRANSFER_TIMED_OUT => TransferStatus::Timeout,
            LIBUSB_TRANSFER_CANCELLED => TransferStatus::Cancelled,
            LIBUSB_TRANSFER_STALL => TransferStatus::Stall,
            LIBUSB_TRANSFER_NO_DEVICE => TransferStatus::NoDevice,
            _ => TransferStatus::Unknown,
        }
    }

    /// Access the buffer of a transfer.
    pub fn buffer(&mut self) -> &'d mut [u8] {
        unsafe {
            slice::from_raw_parts_mut((*self.transfer).buffer, (*self.transfer).length as usize)
        }
    }

    /// Replace the buffer of a transfer.
    pub fn set_buffer(&mut self, buffer: &'d mut [u8]) {
        unsafe {
            (*self.transfer).buffer = buffer.as_mut_ptr();
            (*self.transfer).length = buffer.len() as i32;
            (*self.transfer).actual_length = 0;
        }
    }

    /// Access the slice of the buffer containing actual data received on an IN transfer.
    pub fn actual(&mut self) -> &'d mut [u8] {
        unsafe {
            slice::from_raw_parts_mut(
                (*self.transfer).buffer,
                (*self.transfer).actual_length as usize,
            )
        }
    }
}

impl<'d> Drop for Transfer<'d> {
    fn drop(&mut self) {
        unsafe {
            libusb_free_transfer(self.transfer);
        }
    }
}

/// Internal type holding data touched by libusb completion callback.
struct CallbackData {
    /// Transfers that have completed, but haven't yet been returned from `wait_any`.
    completed: Mutex<VecDeque<*mut libusb_transfer>>,

    /// Signals a completion to avoid race conditions between callback and
    /// `libusb_handle_events_completed`. This is synchronized with the
    /// Mutex above, but can't be included in it because libusb reads it
    /// without the lock held.
    flag: UnsafeCell<c_int>,
}

/// An AsyncGroup manages outstanding asynchronous transfers.
pub struct AsyncGroup<'d> {
    context: &'d Context,

    /// The data touched by the callback, boxed to keep a consistent address if the AsyncGroup
    /// is moved while transfers are active.
    callback_data: Box<CallbackData>,

    /// The set of pending transfers. We need to keep track of them so they can be cancelled on
    /// drop.
    pending: HashSet<*mut libusb_transfer>,
}

/// The libusb transfer completion callback. Careful: libusb may call this on any thread!
extern "C" fn async_group_callback(transfer: *mut libusb_transfer) {
    unsafe {
        let callback_data: &CallbackData = &*((*transfer).user_data as *const CallbackData);
        let mut completed = callback_data.completed.lock().unwrap();
        completed.push_back(transfer);
        *(callback_data.flag.get()) = 1;
    }
}

impl<'d> AsyncGroup<'d> {
    /// Creates an AsyncGroup to process transfers for devices from the given context.
    pub fn new(context: &'d Context) -> AsyncGroup<'d> {
        AsyncGroup {
            context,
            callback_data: Box::new(CallbackData {
                completed: Mutex::new(VecDeque::new()),
                flag: UnsafeCell::new(0),
            }),
            pending: HashSet::new(),
        }
    }

    /// Starts a transfer.
    ///
    /// The Transfer is owned by the AsyncGroup while it is pending, and is
    /// returned from `wait_any` when it completes or fails.
    pub fn submit(&mut self, t: Transfer<'d>) -> Result<()> {
        unsafe {
            (*t.transfer).user_data = &mut *self.callback_data as *mut _ as *mut c_void;
            (*t.transfer).callback = async_group_callback;
        }
        try_unsafe!(libusb_submit_transfer(t.transfer));
        self.pending.insert(t.transfer);
        mem::forget(t);
        Ok(())
    }

    /// Waits for any pending transfer to complete, and return it.
    pub fn wait_any(&mut self) -> Result<Transfer<'d>> {
        if self.pending.is_empty() {
            // Otherwise this function would block forever waiting for a transfer to complete
            return Err(Error::NotFound);
        }

        {
            let transfer;
            loop {
                {
                    let mut completed = self.callback_data.completed.lock().unwrap();
                    if let Some(t) = completed.pop_front() {
                        transfer = t;
                        break;
                    }
                    unsafe { *self.callback_data.flag.get() = 0 };
                }
                try_unsafe!(libusb_handle_events_completed(
                    self.context.as_raw(),
                    self.callback_data.flag.get()
                ));
            }

            if !self.pending.remove(&transfer) {
                panic!("Got a completion for a transfer that wasn't pending");
            }

            Ok(Transfer {
                transfer,
                _handle: PhantomData,
                _buffer: PhantomData,
            })
        }
    }

    /// Cancels all pending transfers.
    ///
    /// Throws away any received data and errors on transfers that have completed, but haven't been
    /// collected by `wait_any`.
    pub fn cancel_all(&mut self) -> Result<()> {
        for &transfer in self.pending.iter() {
            try_unsafe!(libusb_cancel_transfer(transfer))
        }

        while !self.pending.is_empty() {
            self.wait_any()?;
        }

        Ok(())
    }
}

impl<'d> Drop for AsyncGroup<'d> {
    fn drop(&mut self) {
        self.cancel_all().ok();
    }
}
