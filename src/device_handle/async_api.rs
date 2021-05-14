use crate::{DeviceHandle, UsbContext};

use libc::c_void;
use libusb1_sys as ffi;

use std::convert::{TryFrom, TryInto};
use std::marker::{PhantomData, PhantomPinned};
use std::pin::Pin;
use std::ptr::NonNull;

// TODO: Make the Err variant useful
type CbResult = Result<(), i32>;

struct AsyncTransfer<'d, 'b, C: UsbContext, F> {
    ptr: *mut ffi::libusb_transfer,
    buf: &'b mut [u8],
    closure: F,
    _pin: PhantomPinned, // `ptr` holds a ptr to `buf` and `callback`, so we must ensure that we don't move
    _device: PhantomData<&'d DeviceHandle<C>>,
}
impl<'d, 'b, C: UsbContext, F: FnMut(CbResult)> AsyncTransfer<'d, 'b, C, F> {
    #[allow(unused)]
    pub fn new_bulk(
        device: &'d DeviceHandle<C>,
        endpoint: u8,
        buffer: &'b mut [u8],
        callback: F,
        timeout: std::time::Duration,
    ) -> Pin<Box<Self>> {
        // non-isochronous endpoints (e.g. control, bulk, interrupt) specify a value of 0
        let ptr = unsafe { ffi::libusb_alloc_transfer(0) };
        let ptr = NonNull::new(ptr).expect("Could not allocate transfer!");
        let timeout = libc::c_uint::try_from(timeout.as_millis())
            .expect("Duration was too long to fit into a c_uint");

        // Safety: Pinning `result` ensures it doesn't move, but we know that we will
        // want to access its fields mutably, we just don't want its memory location
        // changing (or its fields moving!). So routinely we will unsafely interact with
        // its fields mutably through a shared reference, but this is still sound.
        let result = Box::pin(Self {
            ptr: std::ptr::null_mut(),
            buf: buffer,
            closure: callback,
            _pin: PhantomPinned,
            _device: PhantomData,
        });

        let closure_as_ptr: *mut F = {
            let mut_ref: *const F = &result.closure;
            mut_ref as *mut F
        };
        unsafe {
            ffi::libusb_fill_bulk_transfer(
                ptr.as_ptr(),
                device.as_raw(),
                endpoint,
                result.buf.as_ptr() as *mut u8,
                result.buf.len().try_into().unwrap(),
                Self::transfer_cb,
                closure_as_ptr.cast(),
                timeout,
            )
        };
        result
    }

    // We need to invoke our closure using a c-style function, so we store the closure
    // inside the custom user data field of the transfer struct, and then call the
    // user provided closure from there.
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
        // this should be fine
        let closure = unsafe {
            let closure: *mut F = std::mem::transmute(transfer.user_data);
            &mut *closure
        };

        use ffi::constants::*;
        match transfer.status {
            LIBUSB_TRANSFER_CANCELLED => {
                // Transfer was cancelled, free the transfer
                unsafe { ffi::libusb_free_transfer(transfer) }
            }
            LIBUSB_TRANSFER_COMPLETED => {
                // call user callback
                (*closure)(Ok(()));
            }
            LIBUSB_TRANSFER_ERROR => (*closure)(Err(LIBUSB_TRANSFER_ERROR)),
            LIBUSB_TRANSFER_TIMED_OUT => (*closure)(Err(LIBUSB_TRANSFER_TIMED_OUT)),
            LIBUSB_TRANSFER_STALL => (*closure)(Err(LIBUSB_TRANSFER_STALL)),
            LIBUSB_TRANSFER_NO_DEVICE => (*closure)(Err(LIBUSB_TRANSFER_NO_DEVICE)),
            LIBUSB_TRANSFER_OVERFLOW => unreachable!(),
            _ => panic!("Found an unexpected error value for transfer status"),
        }
    }
}
impl<C: UsbContext, F> AsyncTransfer<'_, '_, C, F> {
    /// Helper function for the Drop impl.
    fn drop_helper(this: Pin<&mut Self>) {
        // Actual drop code goes here.
        let errno = unsafe { ffi::libusb_cancel_transfer(this.ptr) };
        match errno {
            0 | ffi::constants::LIBUSB_ERROR_NOT_FOUND => (),
            errno => {
                log::warn!(
                    "Could not cancel USB transfer. Memory may be leaked. Errno: {}",
                    errno
                )
            }
        }
    }
}

impl<C: UsbContext, F> Drop for AsyncTransfer<'_, '_, C, F> {
    fn drop(&mut self) {
        // We call `drop_helper` because that function represents the actual type
        // semantics that `self` has when being dropped.
        // (see https://doc.rust-lang.org/std/pin/index.html#drop-implementation)
        // Safety: `new_unchecked` is okay because we know this value is never used
        // again after being dropped.
        Self::drop_helper(unsafe { Pin::new_unchecked(self) });
    }
}
