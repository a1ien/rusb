use std::convert::TryInto;
use std::ffi::c_void;
use std::future::Future;
use std::pin::Pin;
use std::ptr::NonNull;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::time::Duration;

use async_trait::async_trait;
use libc::{c_int, c_uint};
use rusb::constants::*;
use rusb::ffi::*;
use rusb::{DeviceHandle, Error, UsbContext};

const LIBUSB_TRANSFER_ACTIVE: c_int = -1;

fn check_transfer_error(status: c_int) -> Result<(), Error> {
    if status < 0 {
        Err(match status {
            LIBUSB_ERROR_NO_DEVICE => Error::NoDevice,
            LIBUSB_ERROR_BUSY => Error::Busy,
            LIBUSB_ERROR_NOT_SUPPORTED => Error::NotSupported,
            LIBUSB_ERROR_INVALID_PARAM => Error::InvalidParam,
            _ => Error::Other,
        })
    } else {
        Ok(())
    }
}

struct WrappedTransfer(NonNull<libusb_transfer>);

// SAFETY: only used behind an `Arc<Mutex<T>>`.
unsafe impl Send for WrappedTransfer {}

struct InnerTransfer<T: UsbContext> {
    transfer: WrappedTransfer,
    _context: T,
    status: c_int,
    actual_length: c_int,
    waker: Option<Waker>,
}

impl<T: UsbContext> InnerTransfer<T> {
    fn new(context: T) -> Result<Self, Error> {
        let transfer = unsafe { libusb_alloc_transfer(0) };

        if transfer.is_null() {
            return Err(Error::NoMem);
        }

        Ok(Self {
            // SAFETY: transfer is not NULL.
            transfer: unsafe { WrappedTransfer(NonNull::new_unchecked(transfer)) },
            _context: context,
            status: LIBUSB_TRANSFER_ACTIVE,
            actual_length: -1,
            waker: None,
        })
    }

    fn as_ptr(&mut self) -> *mut libusb_transfer {
        self.transfer.0.as_ptr()
    }
}

impl<T: UsbContext> Drop for InnerTransfer<T> {
    fn drop(&mut self) {
        // SAFETY: transfer points to a valid libusb_transfer struct.
        unsafe { libusb_free_transfer(self.transfer.0.as_ptr()) };
    }
}

extern "system" fn transfer_finished<T: UsbContext>(transfer_ptr: *mut libusb_transfer) {
    if transfer_ptr.is_null() {
        return;
    }

    // SAFETY: transfer_ptr is not NULL.
    let transfer: &mut libusb_transfer = unsafe { &mut *transfer_ptr };
    let user_data = transfer.user_data;

    if user_data.is_null() {
        return;
    }

    // SAFETY: user_data is not NULL and the only user always passes a valid `Arc<Mutex<InnerTransfer<T>>>`.
    let inner = unsafe { Arc::from_raw(user_data as *mut Mutex<InnerTransfer<T>>) };
    let mut inner = inner.lock().unwrap();

    inner.status = transfer.status;
    inner.actual_length = transfer.actual_length;

    if let Some(waker) = inner.waker.take() {
        waker.wake()
    }
}

/// Represents a cancellation token for the [`Transfer`]. This allows the user to cancel the USB
/// transfer while it is still pending.
#[derive(Clone)]
pub struct CancellationToken<T: UsbContext> {
    inner: Arc<Mutex<InnerTransfer<T>>>,
}

impl<T: UsbContext> CancellationToken<T> {
    /// Asynchronously cancels the pending USB transfer. This function returns immediately, but
    /// this does not indicate that the cancellation is complete. Instead this will unblock the
    /// task awaiting the [`Transfer`] future and cause it to return [`Error::Interrupted`].
    pub fn cancel(&self) {
        let mut inner = self.inner.lock().unwrap();
        let ptr = inner.as_ptr();

        if inner.status == LIBUSB_TRANSFER_ACTIVE {
            // SAFETY: the transfer is guarded by `Arc<Mutex<T>>` and we only cancel the transfer
            // if it is still active.
            unsafe {
                libusb_cancel_transfer(ptr);
            }
        }
    }
}

/// Represents a submitted USB transfer that can be polled until completion, in which case it will
/// return the ownership of the associated buffer. In the transfer got cancelled, this returns
/// [`Error::Interrupted`].
pub struct Transfer<T: UsbContext> {
    inner: Arc<Mutex<InnerTransfer<T>>>,
    _context: T,
    buffer: Arc<Mutex<Option<Box<[u8]>>>>,
}

impl<T: UsbContext> Drop for Transfer<T> {
    fn drop(&mut self) {
        let mut inner = self.inner.lock().unwrap();
        let ptr = inner.as_ptr();

        inner.waker = None;

        if inner.status == LIBUSB_TRANSFER_ACTIVE {
            // SAFETY: the transfer is guarded by `Arc<Mutex<T>>` and we only cancel the transfer
            // if it is still active.
            unsafe {
                libusb_cancel_transfer(ptr);
            }
        }
    }
}

impl<T: UsbContext> Future for Transfer<T> {
    type Output = Result<(Box<[u8]>, usize), (Box<[u8]>, Error)>;

    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut inner = self.inner.lock().unwrap();

        // The transfer has not been completed, cancelled or errored out. Clone the waker and
        // return that the transfer is still pending.
        if inner.status == LIBUSB_TRANSFER_ACTIVE {
            inner.waker = Some(ctx.waker().clone());

            return Poll::Pending;
        }

        // At this point it is safe to claim ownership of the buffer since the transfer_finished
        // callback has been called as the transfer has been completed, cancelled or errored out.
        //
        // In addition, `Future::poll()` should not be called a second time after it returns
        // `Poll::Ready`. Thus, it is safe to panic.
        let buffer = self.buffer.lock().unwrap().take().unwrap();

        // The transfer completed.
        if inner.status == LIBUSB_TRANSFER_COMPLETED {
            return Poll::Ready(Ok((buffer, inner.actual_length as usize)));
        }

        // The transfer has either been cancelled or errored out.
        let e = match inner.status {
            LIBUSB_TRANSFER_TIMED_OUT => Error::Timeout,
            LIBUSB_TRANSFER_CANCELLED => Error::Interrupted,
            _ => Error::Other,
        };

        return Poll::Ready(Err((buffer, e)));
    }
}

impl<T: UsbContext> Transfer<T> {
    /// Constructs a new bulk transfer to transfer data to/from the bulk endpoint with the address
    /// given by the `endpoint` parameter and fills `data` with any data received from the endpoint
    /// or writes the contents of `data` to the endpoint depending on the direction of the
    /// endpoint. The transfer will claim ownership of `data` until the request completes, gets
    /// cancelled or errors out, upon which the ownership of `data` is given back to the task
    /// awaiting this transfer.
    ///
    /// The function blocks up to the amount of time specified by `timeout`. Minimal `timeout` is 1
    /// milliseconds, anything smaller will result in an infinite block.
    ///
    /// If the return value is `Ok((data, n))`, then `data` is populated with `n` bytes of data
    /// received from the endpoint for readable endpoints. Otherwise for writeable endpoints, `n`
    /// bytes of `data` were written to the endpoint.
    ///
    /// ## Errors
    ///
    /// If this function encounters any form of error while fulfilling the transfer request, an
    /// error variant will be returned. If an error variant is returned, no bytes were transferred.
    ///
    /// The errors returned by polling this transfer include:
    ///
    ///  * `InvalidParam` if the endpoint is not an output endpoint.
    ///  * `Timeout` if the transfer timed out.
    ///  * `Interrupted` if the transfer was cancelled.
    ///  * `Pipe` if the endpoint halted.
    ///  * `NoDevice` if the device has been disconnected.
    ///  * `Io` if the transfer encountered an I/O error.
    pub fn new_bulk_transfer(
        device: &DeviceHandle<T>,
        endpoint: u8,
        mut data: Box<[u8]>,
        timeout: Duration,
    ) -> Result<Self, (Box<[u8]>, Error)> {
        let context = device.context().clone();
        let device = unsafe { NonNull::new_unchecked(device.as_raw()) };

        let max_size = data.len() as i32;
        let timeout = timeout.as_millis() as c_uint;

        let mut inner = match InnerTransfer::new(context.clone()) {
            Ok(inner) => inner,
            Err(e) => return Err((data, e)),
        };
        let transfer_ptr = inner.as_ptr();
        let transfer = Arc::new(Mutex::new(inner));

        let result = {
            let state_ptr = Arc::into_raw(transfer.clone()) as *mut c_void;
            let buffer: *mut u8 = data.as_mut_ptr();

            unsafe {
                libusb_fill_bulk_transfer(
                    transfer_ptr,
                    device.as_ptr(),
                    endpoint,
                    buffer,
                    max_size,
                    transfer_finished::<T> as _,
                    state_ptr,
                    timeout,
                );
            }

            unsafe { libusb_submit_transfer(transfer_ptr) }
        };

        if let Err(e) = check_transfer_error(result) {
            return Err((data, e));
        }

        Ok(Self {
            inner: transfer,
            _context: context,
            buffer: Arc::new(Mutex::new(Some(data))),
        })
    }

    /// Construct a new control transfer to transfer data to/from the device using a control
    /// transfer and fills `data` with any data received during the transfer or writes the contents
    /// of `data` to the device depending on the direction of `request_type`. The transfer will
    /// claim ownership of `data` until the request completes, gets cancelled or errors out, upon
    /// which the ownership of `data` is given back to the task awaiting this transfer.
    ///
    /// The parameters `request_type`, `request`, `value` and `index` specify the fields of the
    /// control transfer setup packet (`bmRequestType`, `bmRequest`, `wValue` and `wIndex`
    /// respectively). The values for each of these parameters shall be given in host-endian byte
    /// order. The value for the `request_type` parameter can be built with the helper function
    /// [request_type()](fn.request_type.html). The meaning of the other parameters depends on the
    /// type of control request.
    ///
    /// As these parameters are stored in the first [`LIBUSB_CONTROL_SETUP_SIZE`] bytes of the
    /// control request, the buffer must be at least [`LIBUSB_CONTROL_SETUP_SIZE`] bytes in size.
    ///
    /// The function blocks up to the amount of time specified by `timeout`. Minimal `timeout` is 1
    /// milliseconds, anything smaller will result in an infinite block.
    ///
    /// If the return value is `Ok((data, n))`, then `data` is populated with `n` bytes of data
    /// received from the device for read requests. This data can be found starting at offset
    /// [`LIBUSB_CONTROL_SETUP_SIZE`] in `data`. Otherwise for write requests, `n` bytes of `data`
    /// were written to the device.
    ///
    /// ## Errors
    ///
    /// If this function encounters any form of error while fulfilling the transfer request, an
    /// error variant will be returned. If an error variant is returned, no bytes were transferred.
    ///
    /// The errors returned by this function include:
    ///
    ///  * `InvalidParam` if buffer is not at least [`LIBUSB_CONTROL_SETUP_SIZE`] bytes in size.
    ///  * `Timeout` if the transfer timed out.
    ///  * `Interrupted` if the transfer was cancelled.
    ///  * `Pipe` if the endpoint halted.
    ///  * `NoDevice` if the device has been disconnected.
    ///  * `Io` if the transfer encountered an I/O error.
    ///
    /// [`LIBUSB_CONTROL_SETUP_SIZE`]: rusb::constants::LIBUSB_CONTROL_SETUP_SIZE
    pub fn new_control_transfer(
        device: &DeviceHandle<T>,
        request_type: u8,
        request: u8,
        value: u16,
        index: u16,
        mut data: Box<[u8]>,
        timeout: Duration,
    ) -> Result<Self, (Box<[u8]>, Error)> {
        if data.len() < LIBUSB_CONTROL_SETUP_SIZE {
            return Err((data, Error::InvalidParam));
        }

        let context = device.context().clone();
        // SAFETY: device.as_raw() must be a valid pointer.
        let device = unsafe { NonNull::new_unchecked(device.as_raw()) };

        let max_size: u16 = match (data.len() - LIBUSB_CONTROL_SETUP_SIZE).try_into() {
            Ok(n) => n,
            Err(_) => return Err((data, Error::InvalidParam)),
        };

        let timeout = timeout.as_millis() as c_uint;

        let mut inner = match InnerTransfer::new(context.clone()) {
            Ok(inner) => inner,
            Err(e) => return Err((data, e)),
        };
        let transfer_ptr = inner.as_ptr();
        let transfer = Arc::new(Mutex::new(inner));

        let result = {
            let state_ptr = Arc::into_raw(transfer.clone()) as *mut c_void;
            let buffer: *mut u8 = data.as_mut_ptr();

            // SAFETY: buffer has at least LIBUSB_CONTROL_SETUP_SIZE bytes and is a valid pointer.
            unsafe {
                libusb_fill_control_setup(buffer, request_type, request, value, index, max_size);
            }

            // SAFETY: transfer_ptr, device.as_ptr(), buffer, state_ptr are all valid. This is the
            // only user of transfer_finished and transfer_finished gets state_ptr which is of the
            // type `Arc<Mutex<InnerTransfer<T>>>` as expected. These pointers remain valid until
            // `transfer_finished` gets called.
            unsafe {
                libusb_fill_control_transfer(
                    transfer_ptr,
                    device.as_ptr(),
                    buffer,
                    transfer_finished::<T> as _,
                    state_ptr,
                    timeout,
                );
            }

            // SAFETY: we ensure that transfer_ptr and buffer are valid until completion,
            // cancellation or an error occurs. In addition, as buffer is `Pin`, it is guaranteed
            // to not move around while the transfer is in progress.
            unsafe { libusb_submit_transfer(transfer_ptr) }
        };

        if let Err(e) = check_transfer_error(result) {
            return Err((data, e));
        }

        Ok(Self {
            inner: transfer,
            _context: context,
            buffer: Arc::new(Mutex::new(Some(data))),
        })
    }

    /// Constructs a new interrupt transfer to transfer data to/from the interrupt endpoint with
    /// the address given by the `endpoint` parameter and fills `data` with any data received from
    /// the endpoint or writes the contents of `data` to the endpoint depending on the direction of
    /// the endpoint. The transfer will claim ownership of `data` until the request completes, gets
    /// cancelled or errors out, upon which the ownership of `data` is given back to the task
    /// awaiting this transfer.
    ///
    /// The function blocks up to the amount of time specified by `timeout`. Minimal `timeout` is 1
    /// milliseconds, anything smaller will result in an infinite block.
    ///
    /// If the return value is `Ok((data, n))`, then `data` is populated with `n` bytes of data
    /// received from the endpoint for readable endpoints. Otherwise for writeable endpoints, `n`
    /// bytes of `data` were written to the endpoint.
    ///
    /// ## Errors
    ///
    /// If this function encounters any form of error while fulfilling the transfer request, an
    /// error variant will be returned. If an error variant is returned, no bytes were transferred.
    ///
    /// The errors returned by polling this transfer include:
    ///
    ///  * `InvalidParam` if the endpoint is not an output endpoint.
    ///  * `Timeout` if the transfer timed out.
    ///  * `Interrupted` if the transfer was cancelled.
    ///  * `Pipe` if the endpoint halted.
    ///  * `NoDevice` if the device has been disconnected.
    ///  * `Io` if the transfer encountered an I/O error.
    pub fn new_interrupt_transfer(
        device: &DeviceHandle<T>,
        endpoint: u8,
        mut data: Box<[u8]>,
        timeout: Duration,
    ) -> Result<Self, (Box<[u8]>, Error)> {
        let context = device.context().clone();
        let device = unsafe { NonNull::new_unchecked(device.as_raw()) };

        let max_size = data.len() as i32;
        let timeout = timeout.as_millis() as c_uint;

        let mut inner = match InnerTransfer::new(context.clone()) {
            Ok(inner) => inner,
            Err(e) => return Err((data, e)),
        };
        let transfer_ptr = inner.as_ptr();
        let transfer = Arc::new(Mutex::new(inner));

        let result = {
            let state_ptr = Arc::into_raw(transfer.clone()) as *mut c_void;
            let buffer: *mut u8 = data.as_mut_ptr();

            unsafe {
                libusb_fill_interrupt_transfer(
                    transfer_ptr,
                    device.as_ptr(),
                    endpoint,
                    buffer,
                    max_size,
                    transfer_finished::<T> as _,
                    state_ptr,
                    timeout,
                );
            }

            unsafe { libusb_submit_transfer(transfer_ptr) }
        };

        if let Err(e) = check_transfer_error(result) {
            return Err((data, e));
        }

        Ok(Self {
            inner: transfer,
            _context: context,
            buffer: Arc::new(Mutex::new(Some(data))),
        })
    }

    /// Constructs a [`CancellationToken`] that can be used to cancel the USB transfer.
    pub fn cancellation_token(&self) -> CancellationToken<T> {
        CancellationToken {
            inner: self.inner.clone(),
        }
    }
}

#[async_trait]
pub trait DeviceHandleExt {
    /// Asynchronously reads from a bulk endpoint.
    ///
    /// This function attempts to asynchronously read from the bulk endpoint with the address given
    /// by the `endpoint` parameter and fills `data` with any data received from the endpoint. This
    /// function will claim ownership of `data` until the request completes, gets cancelled or
    /// errors out, upon which the ownership of `data` is given back to the caller of this function.
    ///
    /// The function blocks up to the amount of time specified by `timeout`. Minimal `timeout` is 1
    /// milliseconds, anything smaller will result in an infinite block.
    ///
    /// In case you want the ability to cancel the USB transfer, consider using
    /// [`Transfer::new_bulk_transfer()`] instead.
    ///
    /// If the return value is `Ok((data, n))`, then `data` is populated with `n` bytes of data
    /// received from the endpoint.
    ///
    /// ## Errors
    ///
    /// If this function encounters any form of error while fulfilling the transfer request, an
    /// error variant will be returned. If an error variant is returned, no bytes were read.
    ///
    /// The errors returned by this function include:
    ///
    ///  * `InvalidParam` if the endpoint is not an input endpoint.
    ///  * `Timeout` if the transfer timed out.
    ///  * `Interrupted` if the transfer was cancelled.
    ///  * `Pipe` if the endpoint halted.
    ///  * `NoDevice` if the device has been disconnected.
    ///  * `Io` if the transfer encountered an I/O error.
    async fn read_bulk_async(
        &self,
        endpoint: u8,
        data: Box<[u8]>,
        timeout: Duration,
    ) -> Result<(Box<[u8]>, usize), (Box<[u8]>, Error)>;

    /// Asynchronously reads data using a control transfer.
    ///
    /// This function attempts to asynchronously read data from the device using a control transfer
    /// and fills `data` with any data received during the transfer. This function will claim
    /// ownership of `data` until the request completes, gets cancelled or errors out, upon which
    /// the ownership of `data` is given back to the caller of this function.
    ///
    /// The parameters `request_type`, `request`, `value` and `index` specify the fields of the
    /// control transfer setup packet (`bmRequestType`, `bmRequest`, `wValue` and `wIndex`
    /// respectively). The values for each of these parameters shall be given in host-endian byte
    /// order. The value for the `request_type` parameter can be built with the helper function
    /// [request_type()](fn.request_type.html). The meaning of the other parameters depends on the
    /// type of control request.
    ///
    /// As these parameters are stored in the first [`LIBUSB_CONTROL_SETUP_SIZE`] bytes of the
    /// control request, the buffer must be at least [`LIBUSB_CONTROL_SETUP_SIZE`] bytes in size.
    ///
    /// The function blocks up to the amount of time specified by `timeout`. Minimal `timeout` is 1
    /// milliseconds, anything smaller will result in an infinite block.
    ///
    /// In case you want the ability to cancel the USB transfer, consider using
    /// [`Transfer::new_control_transfer()`] instead.
    ///
    /// If the return value is `Ok((data, n))`, then `data` is populated with `n` bytes of data
    /// received from the device. This data can be found starting at offset
    /// [`LIBUSB_CONTROL_SETUP_SIZE`] in `data`.
    ///
    /// ## Errors
    ///
    /// If this function encounters any form of error while fulfilling the transfer request, an
    /// error variant will be returned. If an error variant is returned, no bytes were read.
    ///
    /// The errors returned by this function include:
    ///
    ///  * `InvalidParam` if the `request_type` does not specify a read transfer.
    ///  * `InvalidParam` if buffer is not at least [`LIBUSB_CONTROL_SETUP_SIZE`] bytes in size.
    ///  * `Timeout` if the transfer timed out.
    ///  * `Interrupted` if the transfer was cancelled.
    ///  * `Pipe` if the endpoint halted.
    ///  * `NoDevice` if the device has been disconnected.
    ///  * `Io` if the transfer encountered an I/O error.
    ///
    /// [`LIBUSB_CONTROL_SETUP_SIZE`]: rusb::constants::LIBUSB_CONTROL_SETUP_SIZE
    async fn read_control_async(
        &self,
        request_type: u8,
        request: u8,
        value: u16,
        index: u16,
        data: Box<[u8]>,
        timeout: Duration,
    ) -> Result<(Box<[u8]>, usize), (Box<[u8]>, Error)>;

    /// Asynchronously reads from an interrupt endpoint.
    ///
    /// This function attempts to asynchronously read from the interrupt endpoint with the address
    /// given by the `endpoint` parameter and fills `data` with any data received from the endpoint.
    /// This function will claim ownership of `data` until the request completes, gets cancelled or
    /// errors out, upon which the ownership of `data` is given back to the caller of this function.
    ///
    /// The function blocks up to the amount of time specified by `timeout`. Minimal `timeout` is 1
    /// milliseconds, anything smaller will result in an infinite block.
    ///
    /// In case you want the ability to cancel the USB transfer, consider using
    /// [`Transfer::new_bulk_transfer()`] instead.
    ///
    /// If the return value is `Ok((data, n))`, then `data` is populated with `n` bytes of data
    /// received from the endpoint.
    ///
    /// ## Errors
    ///
    /// If this function encounters any form of error while fulfilling the transfer request, an
    /// error variant will be returned. If an error variant is returned, no bytes were read.
    ///
    /// The errors returned by this function include:
    ///
    ///  * `InvalidParam` if the endpoint is not an input endpoint.
    ///  * `Timeout` if the transfer timed out.
    ///  * `Interrupted` if the transfer was cancelled.
    ///  * `Pipe` if the endpoint halted.
    ///  * `NoDevice` if the device has been disconnected.
    ///  * `Io` if the transfer encountered an I/O error.
    async fn read_interrupt_async(
        &self,
        endpoint: u8,
        data: Box<[u8]>,
        timeout: Duration,
    ) -> Result<(Box<[u8]>, usize), (Box<[u8]>, Error)>;

    /// Asynchronously writes to a bulk endpoint.
    ///
    /// This function attempts to asynchronously write the contents of `data` to the bulk endpoint
    /// with the address given by the `endpoint` parameter. This function will claim ownership of
    /// `data` until the request completes, gets cancelled or errors out, upon which the ownership
    /// of `data` is given back to the caller of this function.
    ///
    /// The function blocks up to the amount of time specified by `timeout`. Minimal `timeout` is 1
    /// milliseconds, anything smaller will result in an infinite block.
    ///
    /// In case you want the ability to cancel the USB transfer, consider using
    /// [`Transfer::new_bulk_transfer()`] instead.
    ///
    /// If the return value is `Ok((_, n))`, then `n` bytes of `data` were written to the endpoint.
    ///
    /// ## Errors
    ///
    /// If this function encounters any form of error while fulfilling the transfer request, an
    /// error variant will be returned. If an error variant is returned, no bytes were written.
    ///
    /// The errors returned by this function include:
    ///
    ///  * `InvalidParam` if the endpoint is not an output endpoint.
    ///  * `Timeout` if the transfer timed out.
    ///  * `Interrupted` if the transfer was cancelled.
    ///  * `Pipe` if the endpoint halted.
    ///  * `NoDevice` if the device has been disconnected.
    ///  * `Io` if the transfer encountered an I/O error.
    async fn write_bulk_async(
        &self,
        endpoint: u8,
        data: Box<[u8]>,
        timeout: Duration,
    ) -> Result<(Box<[u8]>, usize), (Box<[u8]>, Error)>;

    /// Asynchronously writes data using a control transfer.
    ///
    /// This function attempts to asynchronously write data to the device using a control transfer
    /// and writes the contents of `data` during the transfer. This function will claim ownership
    /// of `data` until the request completes, gets cancelled or errors out, upon which the
    /// ownership of `data` is given back to the caller of this function.
    ///
    /// The parameters `request_type`, `request`, `value` and `index` specify the fields of the
    /// control transfer setup packet (`bmRequestType`, `bmRequest`, `wValue` and `wIndex`
    /// respectively). The values for each of these parameters shall be given in host-endian byte
    /// order. The value for the `request_type` parameter can be built with the helper function
    /// [request_type()](fn.request_type.html). The meaning of the other parameters depends on the
    /// type of control request.
    ///
    /// As these parameters are stored in the first [`LIBUSB_CONTROL_SETUP_SIZE`] bytes of the
    /// control request, the buffer must be at least [`LIBUSB_CONTROL_SETUP_SIZE`] bytes in size.
    /// The actual data must start at offset [`LIBUSB_CONTROL_SETUP_SIZE`].
    ///
    /// The function blocks up to the amount of time specified by `timeout`. Minimal `timeout` is 1
    /// milliseconds, anything smaller will result in an infinite block.
    ///
    /// In case you want the ability to cancel the USB transfer, consider using
    /// [`Transfer::new_control_transfer()`] instead.
    ///
    /// If the return value is `Ok((_, n))`, then `n` bytes have been written to the device.
    ///
    /// ## Errors
    ///
    /// If this function encounters any form of error while fulfilling the transfer request, an
    /// error variant will be returned. If an error variant is returned, no bytes were written.
    ///
    /// The errors returned by this function include:
    ///
    ///  * `InvalidParam` if the `request_type` does not specify a write transfer.
    ///  * `InvalidParam` if buffer is not at least [`LIBUSB_CONTROL_SETUP_SIZE`] bytes in size.
    ///  * `Timeout` if the transfer timed out.
    ///  * `Interrupted` if the transfer was cancelled.
    ///  * `Pipe` if the endpoint halted.
    ///  * `NoDevice` if the device has been disconnected.
    ///  * `Io` if the transfer encountered an I/O error.
    ///
    /// [`LIBUSB_CONTROL_SETUP_SIZE`]: rusb::constants::LIBUSB_CONTROL_SETUP_SIZE
    async fn write_control_async(
        &self,
        request_type: u8,
        request: u8,
        value: u16,
        index: u16,
        data: Box<[u8]>,
        timeout: Duration,
    ) -> Result<(Box<[u8]>, usize), (Box<[u8]>, Error)>;

    /// Asynchronously writes to an interrupt endpoint.
    ///
    /// This function attempts to asynchronously write the contents of `data` to the interrupt
    /// endpoint with the address given by the `endpoint` parameter. This function will claim
    /// ownership of `data` until the request completes, gets cancelled or errors out, upon which
    /// the ownership of `data` is given back to the caller of this function.
    ///
    /// The function blocks up to the amount of time specified by `timeout`. Minimal `timeout` is 1
    /// milliseconds, anything smaller will result in an infinite block.
    ///
    /// In case you want the ability to cancel the USB transfer, consider using
    /// [`Transfer::new_interrupt_transfer()`] instead.
    ///
    /// If the return value is `Ok((_, n))`, then `n` bytes of `data` were written to the endpoint.
    ///
    /// ## Errors
    ///
    /// If this function encounters any form of error while fulfilling the transfer request, an
    /// error variant will be returned. If an error variant is returned, no bytes were written.
    ///
    /// The errors returned by this function include:
    ///
    ///  * `InvalidParam` if the endpoint is not an output endpoint.
    ///  * `Timeout` if the transfer timed out.
    ///  * `Interrupted` if the transfer was cancelled.
    ///  * `Pipe` if the endpoint halted.
    ///  * `NoDevice` if the device has been disconnected.
    ///  * `Io` if the transfer encountered an I/O error.
    async fn write_interrupt_async(
        &self,
        endpoint: u8,
        data: Box<[u8]>,
        timeout: Duration,
    ) -> Result<(Box<[u8]>, usize), (Box<[u8]>, Error)>;
}

#[async_trait]
impl<T: UsbContext> DeviceHandleExt for DeviceHandle<T> {
    async fn read_bulk_async(
        &self,
        endpoint: u8,
        data: Box<[u8]>,
        timeout: Duration,
    ) -> Result<(Box<[u8]>, usize), (Box<[u8]>, Error)> {
        if endpoint & LIBUSB_ENDPOINT_DIR_MASK != LIBUSB_ENDPOINT_IN {
            return Err((data, Error::InvalidParam));
        }

        let transfer = Transfer::new_bulk_transfer(self, endpoint, data, timeout)?;

        Ok(transfer.await?)
    }

    async fn read_control_async(
        &self,
        request_type: u8,
        request: u8,
        value: u16,
        index: u16,
        data: Box<[u8]>,
        timeout: Duration,
    ) -> Result<(Box<[u8]>, usize), (Box<[u8]>, Error)> {
        if request_type & LIBUSB_ENDPOINT_DIR_MASK != LIBUSB_ENDPOINT_IN {
            return Err((data, Error::InvalidParam));
        }

        let transfer = Transfer::new_control_transfer(
            self,
            request_type,
            request,
            value,
            index,
            data,
            timeout,
        )?;

        Ok(transfer.await?)
    }

    async fn read_interrupt_async(
        &self,
        endpoint: u8,
        data: Box<[u8]>,
        timeout: Duration,
    ) -> Result<(Box<[u8]>, usize), (Box<[u8]>, Error)> {
        if endpoint & LIBUSB_ENDPOINT_DIR_MASK != LIBUSB_ENDPOINT_IN {
            return Err((data, Error::InvalidParam));
        }

        let transfer = Transfer::new_interrupt_transfer(self, endpoint, data, timeout)?;

        Ok(transfer.await?)
    }

    async fn write_bulk_async(
        &self,
        endpoint: u8,
        data: Box<[u8]>,
        timeout: Duration,
    ) -> Result<(Box<[u8]>, usize), (Box<[u8]>, Error)> {
        if endpoint & LIBUSB_ENDPOINT_DIR_MASK != LIBUSB_ENDPOINT_OUT {
            return Err((data, Error::InvalidParam));
        }

        let transfer = Transfer::new_bulk_transfer(self, endpoint, data, timeout)?;

        Ok(transfer.await?)
    }

    async fn write_control_async(
        &self,
        request_type: u8,
        request: u8,
        value: u16,
        index: u16,
        data: Box<[u8]>,
        timeout: Duration,
    ) -> Result<(Box<[u8]>, usize), (Box<[u8]>, Error)> {
        if request_type & LIBUSB_ENDPOINT_DIR_MASK != LIBUSB_ENDPOINT_OUT {
            return Err((data, Error::InvalidParam));
        }

        let transfer = Transfer::new_control_transfer(
            self,
            request_type,
            request,
            value,
            index,
            data,
            timeout,
        )?;

        Ok(transfer.await?)
    }

    async fn write_interrupt_async(
        &self,
        endpoint: u8,
        data: Box<[u8]>,
        timeout: Duration,
    ) -> Result<(Box<[u8]>, usize), (Box<[u8]>, Error)> {
        if endpoint & LIBUSB_ENDPOINT_DIR_MASK != LIBUSB_ENDPOINT_OUT {
            return Err((data, Error::InvalidParam));
        }

        let transfer = Transfer::new_interrupt_transfer(self, endpoint, data, timeout)?;

        Ok(transfer.await?)
    }
}
