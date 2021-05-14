use crate::{DeviceHandle, UsbContext};

use libc::c_void;
use libusb1_sys as ffi;

use std::convert::{TryFrom, TryInto};
use std::marker::{PhantomData, PhantomPinned};
use std::pin::Pin;
use std::ptr::NonNull;

struct AsyncTransfer<'d, 'b, C: UsbContext, F: FnMut()> {
    ptr: *mut ffi::libusb_transfer,
    buf: &'b mut [u8],
    closure: F,
    _pin: PhantomPinned, // `ptr` holds a ptr to `buf` and `callback`, so we must ensure that we don't move
    _device: PhantomData<&'d DeviceHandle<C>>,
}
impl<'d, 'b, C: UsbContext, F: FnMut()> AsyncTransfer<'d, 'b, C, F> {
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
                closure_as_ptr as *mut c_void,
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

        // TODO: check some stuff

        // call user callback
        (*closure)();
    }
}
