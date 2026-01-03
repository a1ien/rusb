mod bulk;
mod control;
mod interrupt;
mod isochronous;
mod ops;

use std::{
    convert::TryInto,
    future::Future,
    ptr::NonNull,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    task::{Poll, Waker},
};

pub use bulk::BulkTransfer;
pub use control::{ControlTransfer, RawControlTransfer};
pub use interrupt::InterruptTransfer;
pub use isochronous::{IsoBufIter, IsochronousBuffer, IsochronousTransfer};
use rusb::{
    constants::LIBUSB_ERROR_BUSY,
    ffi::{
        self,
        constants::{
            LIBUSB_ERROR_INVALID_PARAM, LIBUSB_ERROR_NOT_SUPPORTED, LIBUSB_ERROR_NO_DEVICE,
            LIBUSB_TRANSFER_CANCELLED, LIBUSB_TRANSFER_COMPLETED, LIBUSB_TRANSFER_ERROR,
            LIBUSB_TRANSFER_NO_DEVICE, LIBUSB_TRANSFER_OVERFLOW, LIBUSB_TRANSFER_STALL,
            LIBUSB_TRANSFER_TIMED_OUT,
        },
        libusb_transfer,
    },
    DeviceHandle,
};

use crate::{
    error::{Error, Result},
    transfer::ops::{CompleteTransfer, FillTransfer, SingleBufferTransfer},
    AsyncUsbContext,
};

/// Generic transfer.
///
/// Rather than interacting with this type directly, choose to use
/// the convenience aliases provided for each transfer type.
#[derive(Debug)]
pub struct Transfer<C, K>
where
    C: AsyncUsbContext,
{
    dev_handle: Arc<DeviceHandle<C>>,
    endpoint: u8,
    ptr: NonNull<ffi::libusb_transfer>,
    buffer: Vec<u8>,
    kind: K,
    state: TransferState,
}

impl<C, K> Transfer<C, K>
where
    C: AsyncUsbContext,
{
    /// This is step 1 of async API.
    fn alloc(
        dev_handle: Arc<DeviceHandle<C>>,
        endpoint: u8,
        buffer: Vec<u8>,
        kind: K,
        iso_packets: libc::c_int,
    ) -> Result<Self> {
        let Some(ptr) = NonNull::new(unsafe { ffi::libusb_alloc_transfer(iso_packets) }) else {
            return Err(Error::TransferAlloc);
        };

        Ok(Self {
            dev_handle,
            endpoint,
            ptr,
            buffer,
            kind,
            state: TransferState::Allocated,
        })
    }

    /// Step 3 of async API
    fn submit(&mut self) -> Result<()> {
        let errno = unsafe { ffi::libusb_submit_transfer(self.ptr.as_ptr()) };

        match errno {
            0 => Ok(()),
            LIBUSB_ERROR_NO_DEVICE => Err(Error::Disconnected),
            LIBUSB_ERROR_BUSY => {
                unreachable!("We shouldn't be calling submit on transfers already submitted!")
            }
            LIBUSB_ERROR_NOT_SUPPORTED => Err(Error::Other("Transfer not supported")),
            LIBUSB_ERROR_INVALID_PARAM => {
                Err(Error::Other("Transfer size bigger than OS supports"))
            }
            _ => Err(Error::Errno("Error while submitting transfer: ", errno)),
        }
    }

    /// Part of step 4 of async API.
    ///
    /// When event handling is being performed and this transfer completes,
    /// this function gets called.
    ///
    /// This function handles two cases.
    /// 1) It notifies the async runtime through the provided waker that this transfer completed.
    ///    This can happen mean that the transfer was successful, errored out or, on Darwin based systems,
    ///    that it got cancelled because another transfer on the same endpoint got cancelled.
    ///
    /// 2) It frees the transfer if it was cancelled by dropping it while it was still pending.
    ///    Because it was dropped, the transfer won't be used anywhere else anymore.
    extern "system" fn transfer_cb(transfer: *mut ffi::libusb_transfer) {
        // SAFETY: Transfer is still valid because libusb just completed
        //         it but we haven't told anyone yet. `user_data` remains
        //         valid because it is only freed here or when the transfer gets
        //         dropped.
        unsafe {
            let transfer = &mut *transfer;

            let user_data = &*transfer.user_data.cast::<TransferUserData>();

            // Check that the transfer was cancelled.
            let transfer_cancelled = transfer.status == LIBUSB_TRANSFER_CANCELLED;

            // Did this transfer trigger its own cancellation (was it dropped)?
            //
            // This is important on Darwin based systems where cancelling a transfer will
            // also cancel all other transfers on the same endpoint.
            //
            // See: <https://libusb.sourceforge.io/api-1.0/group__libusb__asyncio.html#ga685eb7731f9a0593f75beb99727bbe54>.
            let cancelled_itself = user_data.cancelled_itself.load(Ordering::SeqCst);

            // This is true only when a submitted transfer was dropped,
            // so destroying it is fine here as it will never get accessed again.
            //
            // Otherwise, even if the transfer is cancelled, we'll wake the future
            // associated with it so that it gets polled to completion (and errors out).
            //
            // The transfer will then get freed on drop.
            if transfer_cancelled && cancelled_itself {
                Self::free(transfer);
            } else {
                user_data.waker.wake_by_ref();
            }
        };
    }

    fn transfer(&self) -> &ffi::libusb_transfer {
        // SAFETY: Transfer remains valid as long as self.
        unsafe { self.ptr.as_ref() }
    }

    fn cancel(&mut self) {
        // SAFETY: Transfer remains valid as long as self.
        unsafe {
            ffi::libusb_cancel_transfer(self.ptr.as_ptr());

            // Take note that this transfer cancelled itself.
            // This is important on Darwin based systems since cancelling a transfer will cancel
            // all other transfers on the same endpoint.
            //
            // See: <https://libusb.sourceforge.io/api-1.0/group__libusb__asyncio.html#ga685eb7731f9a0593f75beb99727bbe54>.
            let user_data = &*self.transfer().user_data.cast::<TransferUserData>();
            user_data.cancelled_itself.store(true, Ordering::SeqCst);
        };
    }

    /// Frees the transfer as well as dropping the user data.
    unsafe fn free(transfer: *mut libusb_transfer) {
        let transfer = &mut *transfer;
        let _ = Box::from_raw(transfer.user_data.cast::<TransferUserData>());
        ffi::libusb_free_transfer(transfer);
    }
}

impl<C, K> Transfer<C, K>
where
    C: AsyncUsbContext,
    Self: CompleteTransfer,
{
    /// The other part of step 4 of the async API.
    ///
    /// Checks the status transfer and returns the output on success.
    fn complete(&mut self) -> Result<<Self as CompleteTransfer>::Output> {
        let err = match self.transfer().status {
            LIBUSB_TRANSFER_COMPLETED => return self.swap_buffer(Vec::new()),
            LIBUSB_TRANSFER_CANCELLED => Error::Cancelled,
            LIBUSB_TRANSFER_ERROR => Error::Other("Error occurred during transfer execution"),
            LIBUSB_TRANSFER_TIMED_OUT => {
                unreachable!("We are using timeout=0 which means no timeout")
            }
            LIBUSB_TRANSFER_STALL => Error::Stall,
            LIBUSB_TRANSFER_NO_DEVICE => Error::Disconnected,
            LIBUSB_TRANSFER_OVERFLOW => Error::Overflow,
            _ => panic!("Found an unexpected error value for transfer status"),
        };
        Err(err)
    }

    /// Replaces the internal transfer buffer so it can be consumed and
    /// the output returned to the caller.
    ///
    /// Prerequisite: self.buffer ans self.ptr are both correctly set
    fn swap_buffer(&mut self, buffer: Vec<u8>) -> Result<<Self as CompleteTransfer>::Output> {
        debug_assert!(self.transfer().length >= self.transfer().actual_length);
        let data = std::mem::replace(&mut self.buffer, buffer);
        let output = self.consume_buffer(data)?;

        // Update transfer struct for new buffer
        let transfer_struct = unsafe { self.ptr.as_mut() };
        transfer_struct.actual_length = 0; // TODO: Is this necessary?
        transfer_struct.buffer = self.buffer.as_mut_ptr();
        transfer_struct.length = self.buffer.capacity().try_into().unwrap();

        Ok(output)
    }
}

// Transfer kinds are not complex types that should require pin projections.
// It's thus much simpler to require that they implement [`Unpin`],
// thus allowing the entire [`Transfer`] to be [`Unpin`].
impl<C, K> Future for Transfer<C, K>
where
    C: AsyncUsbContext,
    K: Unpin,
    Self: CompleteTransfer,
{
    type Output = Result<<Self as CompleteTransfer>::Output>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Self::Output> {
        match self.state {
            // Fill transfer
            TransferState::Allocated => {
                self.fill(cx.waker().clone())?;
                self.state = TransferState::Filled;
                // Re-poll to submit the transfer
                self.poll(cx)
            }
            // Submit transfer and pend until event handling
            // calls the transfer callback that wakes us.
            TransferState::Filled => {
                self.submit()?;
                self.state = TransferState::Submitted;
                Poll::Pending
            }
            // Complete transfer.
            TransferState::Submitted => {
                self.state = TransferState::Completed;
                Poll::Ready(self.complete())
            }
            // The transfer was polled after already completing
            // but without calling `renew()`.
            TransferState::Completed => Poll::Ready(Err(Error::AlreadyCompleted)),
        }
    }
}

/// Step 5 of the async API.
impl<C, K> Drop for Transfer<C, K>
where
    C: AsyncUsbContext,
{
    fn drop(&mut self) {
        match self.state {
            // If the transfer was submitted, we cancel it instead of dropping it.
            //
            // The transfer callback function ([`Transfer::transfer_cb`]) will
            // then free the transfer if it was cancelled. That's safe since
            // we're dropping the transfer here, so nothing else will access
            // it after it's freed in the callback.
            //
            // NOTE: On Darwin based systems this would cancel all transfers on the endpoint.
            //
            // See: <https://libusb.sourceforge.io/api-1.0/group__libusb__asyncio.html#ga685eb7731f9a0593f75beb99727bbe54>.
            TransferState::Submitted => self.cancel(),
            // The transfer was not submitted, so we can safely free it.
            TransferState::Allocated | TransferState::Filled | TransferState::Completed => unsafe {
                Self::free(self.ptr.as_ptr())
            },
        }
    }
}

/// SAFETY: The inner transfer pointer gets a fixed address
///         and the other parts of [`Transfer`] are [`Send`].
unsafe impl<C, K> Send for Transfer<C, K>
where
    C: AsyncUsbContext,
    K: Send,
{
}

/// SAFETY: The inner transfer pointer is only mutated through
///         a mutable reference to [`Transfer`] and the other
///         parts of [`Transfer`] are [`Sync`].
unsafe impl<C, K> Sync for Transfer<C, K>
where
    C: AsyncUsbContext,
    K: Sync,
{
}

/// Type that encapsulates user data passed to the
/// transfer completion callback.
struct TransferUserData {
    waker: Waker,
    cancelled_itself: AtomicBool,
}

impl TransferUserData {
    fn new(waker: Waker) -> Self {
        Self {
            waker,
            cancelled_itself: AtomicBool::new(false),
        }
    }
}

#[derive(Debug)]
enum TransferState {
    Allocated,
    Filled,
    Submitted,
    Completed,
}
