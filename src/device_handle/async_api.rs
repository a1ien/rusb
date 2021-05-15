use crate::{DeviceHandle, UsbContext};

use libc::c_void;
use libusb1_sys as ffi;
use thiserror::Error;

use std::convert::{TryFrom, TryInto};
use std::marker::{PhantomData, PhantomPinned};
use std::pin::Pin;
use std::ptr::NonNull;
use std::time::Duration;

pub type CbResult<'a> = Result<&'a [u8], TransferError>;

#[derive(Error, Debug)]
pub enum TransferError {
    #[error("Transfer timed out")]
    Timeout,
    #[error("Transfer is stalled")]
    Stall,
    #[error("Device was disconnected")]
    Disconnected,
    #[error("Other Error: {0}")]
    Other(&'static str),
    #[error("{0}ERRNO: {1}")]
    Errno(&'static str, i32),
}

pub struct AsyncTransfer<'d, 'b, C: UsbContext, F> {
    ptr: NonNull<ffi::libusb_transfer>,
    closure: F,
    _pin: PhantomPinned, // `ptr` holds a ptr to `closure`, so mark !Unpin
    _device: PhantomData<&'d DeviceHandle<C>>,
    _buf: PhantomData<&'b mut [u8]>,
}
// TODO: should CbResult lifetime be different from 'b?
impl<'d, 'b, C: 'd + UsbContext, F: FnMut(CbResult<'b>) + Send> AsyncTransfer<'d, 'b, C, F> {
    #[allow(unused)]
    pub fn new_bulk(
        device: &'d DeviceHandle<C>,
        endpoint: u8,
        buffer: &'b mut [u8],
        callback: F,
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
            closure: callback,
            _pin: PhantomPinned,
            _device: PhantomData,
            _buf: PhantomData,
        });

        unsafe {
            // This casting, and passing it to the transfer struct, relies on
            // the pointer being a regular pointer and not a fat pointer.
            // Also, closure will be invoked from whatever thread polls, which
            // may be different from the current thread. So it must be `Send`.
            // Also, although many threads at once may poll concurrently, only
            // one will actually ever execute the transfer at a time, so we do
            // not need to worry about simultaneous writes to the buffer
            let closure_as_ptr: *mut F = {
                let ptr: *const F = &result.closure;
                ptr as *mut F
            };
            // Step 2 of async api
            ffi::libusb_fill_bulk_transfer(
                ptr.as_ptr(),
                device.as_raw(),
                endpoint,
                buffer.as_ptr() as *mut u8,
                buffer.len().try_into().unwrap(),
                Self::transfer_cb,
                closure_as_ptr.cast(),
                timeout,
            )
        };
        result
    }

    /// Submits a transfer to libusb's event engine.
    /// # Panics
    /// A transfer should not be double-submitted! Only re-submit after a submission has
    /// returned Err, or the callback has gotten an Err.
    // Step 3 of async API
    #[allow(unused)]
    pub fn submit(self: &mut Pin<Box<Self>>) -> Result<(), TransferError> {
        let errno = unsafe { ffi::libusb_submit_transfer(self.ptr.as_ptr()) };
        use ffi::constants::*;
        match errno {
            0 => Ok(()),
            LIBUSB_ERROR_BUSY => {
                panic!("Do not double-submit a transfer!")
            }
            LIBUSB_ERROR_NOT_SUPPORTED => Err(TransferError::Other("Unsupported transfer!")),
            LIBUSB_ERROR_INVALID_PARAM => Err(TransferError::Other("Transfer size too large!")),
            LIBUSB_ERROR_NO_DEVICE => Err(TransferError::Disconnected),
            _ => Err(TransferError::Errno("Unable to submit transfer. ", errno)),
        }
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

        // sanity
        debug_assert_eq!(
            std::mem::size_of::<*mut F>(),
            std::mem::size_of::<*mut c_void>(),
        );
        // Safety: The pointer shouldn't be a fat pointer, and should be valid, so
        // this should be sound
        let closure = unsafe {
            let closure: *mut F = std::mem::transmute(transfer.user_data);
            &mut *closure
        };

        use ffi::constants::*;
        match transfer.status {
            LIBUSB_TRANSFER_CANCELLED => {
                // Step 5 of async API: Transfer was cancelled, free the transfer
                unsafe { ffi::libusb_free_transfer(transfer) }
            }
            LIBUSB_TRANSFER_COMPLETED => {
                debug_assert!(transfer.length >= transfer.actual_length); // sanity
                let data = unsafe {
                    std::slice::from_raw_parts(transfer.buffer, transfer.actual_length as usize)
                };
                (*closure)(Ok(data));
            }
            LIBUSB_TRANSFER_ERROR => (*closure)(Err(TransferError::Other(
                "Error occurred during transfer execution",
            ))),
            LIBUSB_TRANSFER_TIMED_OUT => {
                (*closure)(Err(TransferError::Timeout));
            }
            LIBUSB_TRANSFER_STALL => (*closure)(Err(TransferError::Stall)),
            LIBUSB_TRANSFER_NO_DEVICE => (*closure)(Err(TransferError::Disconnected)),
            LIBUSB_TRANSFER_OVERFLOW => unreachable!(),
            _ => panic!("Found an unexpected error value for transfer status"),
        }
    }
}
impl<C: UsbContext, F> AsyncTransfer<'_, '_, C, F> {
    /// Helper function for the Drop impl.
    fn drop_helper(self: Pin<&mut Self>) {
        // Actual drop code goes here.
        let transfer_ptr = self.ptr.as_ptr();
        let errno = unsafe { ffi::libusb_cancel_transfer(transfer_ptr) };
        match errno {
            0 | ffi::constants::LIBUSB_ERROR_NOT_FOUND => (),
            errno => {
                log::warn!(
                    "Could not cancel USB transfer. Memory may be leaked. Errno: {}, Error message: {}",
                    errno, unsafe{std::ffi::CStr::from_ptr( ffi::libusb_strerror(errno))}.to_str().unwrap()
                )
            }
        }
    }
}

impl<C: UsbContext, F> Drop for AsyncTransfer<'_, '_, C, F> {
    fn drop(&mut self) {
        // We call `drop_helper` because that function represents the actualsemantics
        // that `self` has when being dropped.
        // (see https://doc.rust-lang.org/std/pin/index.html#drop-implementation)
        // Safety: `new_unchecked` is okay because we know this value is never used
        // again after being dropped.
        Self::drop_helper(unsafe { Pin::new_unchecked(self) });
    }
}

/// Polls for transfers and executes their callbacks. Will block until the
/// given timeout, or return immediately if timeout is zero.
pub fn poll_transfers(ctx: &impl UsbContext, timeout: Duration) {
    let timeval = libc::timeval {
        tv_sec: timeout.as_secs() as i64,
        tv_usec: timeout.subsec_millis() as i64,
    };
    unsafe {
        ffi::libusb_handle_events_timeout_completed(
            ctx.as_raw(),
            std::ptr::addr_of!(timeval),
            std::ptr::null_mut(),
        )
    };
}
