use rusb::ffi::{self, constants::*};

use std::convert::TryInto;
use std::ptr::NonNull;

use std::sync::atomic::{AtomicBool, Ordering};

mod error;
mod pool;

use error::{Error, Result};

pub use pool::TransferPool;

// Transfer metadata: passed to user data and comes back in the callback
// so we can identify the transfer.
pub enum TransferType {
    Bulk { endpoint: u8 },
    Control { request_type: u8, request: u8, value: u16, index: u16 },
    Isochronous { endpoint: u8, num_packets: u32 },
    Interrupt { endpoint: u8 },
}

struct Transfer {
    ptr: NonNull<ffi::libusb_transfer>,
    transfer_type: TransferType,
    buffer: Vec<u8>,
}

// Timeout of 0 means no timeout.
const TRANSFER_TIMEOUT: u32 = 0;

impl Transfer {
    // Invariant: Caller must ensure `device` outlives this transfer
    unsafe fn new(
        device: *mut ffi::libusb_device_handle,
        transfer_type: TransferType,
        mut buffer: Vec<u8>,
    ) -> Self {
        // This is step 1 of async API

        let raw_ptr = match transfer_type {
            TransferType::Isochronous { num_packets, .. } => ffi::libusb_alloc_transfer(num_packets as i32),
            // non-isochronous endpoints (e.g. control, bulk, interrupt) specify a value of 0
            _ => ffi::libusb_alloc_transfer(0),
        };

        let ptr = NonNull::new(raw_ptr).expect("Failed to allocate transfer");

        // Save the tranfer metadata in the user data pointer
        // TODO: Do we need to do this? Maybe it can be reconstructed from the other data in the callback?
        // Is the atomic bool we removed necessary?
        //let user_data = Box::into_raw(Box::new(transfer_type)).cast::<libc::c_void>();
        let user_data = Box::into_raw(Box::new(AtomicBool::new(false))).cast::<libc::c_void>();

        let length = match transfer_type {
            TransferType::Control { .. } => buffer.len(),
            TransferType::Bulk { endpoint } | TransferType::Interrupt { endpoint } | TransferType::Isochronous { endpoint, .. } => {
                if endpoint & LIBUSB_ENDPOINT_DIR_MASK == LIBUSB_ENDPOINT_OUT {
                    // for OUT endpoints: the currently valid data in the buffer
                    buffer.len()
                } else {
                    // for IN endpoints: the full capacity
                    buffer.capacity()
                }
            }
        }.try_into().unwrap();

        match transfer_type {
            TransferType::Bulk { endpoint } => {
                ffi::libusb_fill_bulk_transfer(
                    ptr.as_ptr(),
                    device,
                    endpoint,
                    buffer.as_mut_ptr(),
                    length,
                    Self::transfer_cb,
                    user_data,
                    TRANSFER_TIMEOUT,
                );
            },
            TransferType::Interrupt { endpoint } => {
                ffi::libusb_fill_interrupt_transfer(
                    ptr.as_ptr(),
                    device,
                    endpoint,
                    buffer.as_mut_ptr(),
                    length,
                    Self::transfer_cb,
                    user_data,
                    TRANSFER_TIMEOUT,
                );
            },
            TransferType::Control { request_type, request, value, index} => {
                // Extend the buffer to fit the setup data
                //buffer.resize(buffer.len() + LIBUSB_CONTROL_SETUP_SIZE, 0);
                // TODO: Figure out the correct behavior when the original buffer is not empty,
                // and make sure that we update the length once filled. (and investigate boxed slice)
                buffer.reserve(LIBUSB_CONTROL_SETUP_SIZE);

                ffi::libusb_fill_control_setup(
                    buffer.as_mut_ptr(),
                    request_type,
                    request,
                    value,
                    index,
                    length as u16,
                );

                ffi::libusb_fill_control_transfer(
                    ptr.as_ptr(),
                    device,
                    buffer.as_mut_ptr(),
                    Self::transfer_cb,
                    user_data,
                    TRANSFER_TIMEOUT,
                );
            },
            TransferType::Isochronous { endpoint, num_packets } => {
                ffi::libusb_fill_iso_transfer(
                    ptr.as_ptr(),
                    device,
                    endpoint,
                    buffer.as_mut_ptr(),
                    length,
                    num_packets as i32, //btw why is this signed? shouldn't have a negative number of packets?
                    Self::transfer_cb,
                    user_data,
                    TRANSFER_TIMEOUT,
                );
                ffi::libusb_set_iso_packet_lengths(ptr.as_ptr(), (length / num_packets as i32) as u32);
            }
        };

        Self {
            ptr,
            transfer_type,
            buffer,
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
        let completed = unsafe {
            let transfer = &mut *transfer;
            &*transfer.user_data.cast::<AtomicBool>()
        };
        completed.store(true, Ordering::SeqCst);
    }

    fn transfer(&self) -> &ffi::libusb_transfer {
        // Safety: transfer remains valid as long as self
        unsafe { self.ptr.as_ref() }
    }

    fn completed_flag(&self) -> &AtomicBool {
        // Safety: transfer and user_data remain valid as long as self
        unsafe { &*self.transfer().user_data.cast::<AtomicBool>() }
    }

    /// Prerequisite: self.buffer ans self.ptr are both correctly set
    fn swap_buffer(&mut self, new_buf: Vec<u8>) -> Vec<u8> {
        let transfer_struct = unsafe { self.ptr.as_mut() };

        let data = std::mem::replace(&mut self.buffer, new_buf);

        // Update transfer struct for new buffer
        transfer_struct.actual_length = 0; // TODO: Is this necessary?
        transfer_struct.buffer = self.buffer.as_mut_ptr();
        transfer_struct.length = self.buffer.capacity() as i32;

        data
    }

    // Step 3 of async API
    fn submit(&mut self) -> Result<()> {
        self.completed_flag().store(false, Ordering::SeqCst);
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

    fn cancel(&mut self) {
        unsafe {
            ffi::libusb_cancel_transfer(self.ptr.as_ptr());
        }
    }

    fn handle_completed(&mut self) -> Result<Vec<u8>> {
        assert!(self.completed_flag().load(Ordering::Relaxed));
        let err = match self.transfer().status {
            LIBUSB_TRANSFER_COMPLETED => {
                debug_assert!(self.transfer().length >= self.transfer().actual_length);
                unsafe {
                    self.buffer.set_len(self.transfer().actual_length as usize);
                }
                let data = self.swap_buffer(Vec::new());
                return Ok(data);
            }
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
}

/// Invariant: transfer must not be pending
impl Drop for Transfer {
    fn drop(&mut self) {
        unsafe {
            drop(Box::from_raw(self.transfer().user_data));
            ffi::libusb_free_transfer(self.ptr.as_ptr());
        }
    }
}
