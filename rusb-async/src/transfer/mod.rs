mod bulk;
mod control;
mod interrupt;
mod isochronous;
mod ops;

use std::{
    convert::TryInto,
    future::Future,
    ptr::NonNull,
    sync::Arc,
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
    },
    DeviceHandle, UsbContext,
};

use crate::{
    error::{Error, Result},
    transfer::ops::{CompleteTransfer, FillTransfer, SingleBufferTransfer},
};

#[derive(Debug)]
pub struct Transfer<C, K>
where
    C: UsbContext,
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
    C: UsbContext,
{
    /// This is step 1 of async API.
    fn alloc(
        dev_handle: Arc<DeviceHandle<C>>,
        endpoint: u8,
        buffer: Vec<u8>,
        kind: K,
        iso_packets: libc::c_int,
    ) -> Result<Self> {
        // non-isochronous endpoints (e.g. control, bulk, interrupt) specify a value of 0

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

    // Step 3 of async API
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

    // Part of step 4 of async API the transfer is finished being handled when
    // `poll()` is called.
    extern "system" fn transfer_cb(transfer: *mut ffi::libusb_transfer) {
        // Safety: transfer is still valid because libusb just completed
        // it but we haven't told anyone yet. user_data remains valid
        // because it is freed only with the transfer.
        // After the store to completed, these may no longer be valid if
        // the polling thread freed it after seeing it completed.
        unsafe {
            let transfer = &mut *transfer;

            if transfer.status == LIBUSB_TRANSFER_CANCELLED {
                ffi::libusb_free_transfer(transfer);
            } else {
                Box::from_raw(transfer.user_data.cast::<Waker>()).wake();
            }
        };
    }

    fn transfer(&self) -> &ffi::libusb_transfer {
        // Safety: transfer remains valid as long as self
        unsafe { self.ptr.as_ref() }
    }

    fn cancel(&mut self) {
        unsafe { ffi::libusb_cancel_transfer(self.ptr.as_ptr()) };
    }
}

impl<C, K> Transfer<C, K>
where
    C: UsbContext,
    Self: CompleteTransfer,
{
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
// It's thus much simpler to require that they implement [`Unpin`].
impl<C, K> Future for Transfer<C, K>
where
    C: UsbContext,
    K: Unpin,
    Self: CompleteTransfer,
{
    type Output = Result<<Self as CompleteTransfer>::Output>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Self::Output> {
        loop {
            break match self.state {
                TransferState::Allocated => {
                    self.fill(cx.waker().clone())?;
                    self.state = TransferState::Filled;
                    continue;
                }
                TransferState::Filled => {
                    self.submit()?;
                    self.state = TransferState::Submitted;
                    Poll::Pending
                }
                TransferState::Submitted => {
                    self.state = TransferState::Completed;
                    Poll::Ready(self.complete())
                }
                // NOTE: Maybe return an error here instead?
                //       Might make sense since a transfer
                //       could be refilled and polled again.
                TransferState::Completed => Poll::Pending,
            };
        }
    }
}

/// Step 5 of the async API.
impl<C, K> Drop for Transfer<C, K>
where
    C: UsbContext,
{
    fn drop(&mut self) {
        match self.state {
            TransferState::Submitted => self.cancel(),
            TransferState::Allocated | TransferState::Filled | TransferState::Completed => unsafe {
                ffi::libusb_free_transfer(self.ptr.as_ptr());
            },
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
