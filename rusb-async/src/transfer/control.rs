use std::{sync::Arc, task::Waker};

use rusb::{constants::LIBUSB_CONTROL_SETUP_SIZE, ffi, DeviceHandle};

use crate::{
    error::{Error, Result},
    transfer::{FillTransfer, SingleBufferTransfer, Transfer, TransferState, TransferUserData},
    AsyncUsbContext,
};

/// Control transfer.
pub type ControlTransfer<C> = Transfer<C, Control>;
/// Raw control transfer.
pub type RawControlTransfer<C> = Transfer<C, RawControl>;

/// Control transfer kind.
#[allow(missing_copy_implementations)]
#[derive(Debug)]
pub struct Control {
    request_type: u8,
    request: u8,
    value: u16,
    index: u16,
}

impl<C> ControlTransfer<C>
where
    C: AsyncUsbContext,
{
    /// Constructs and allocates a new [`ControlTransfer`].
    ///
    /// # Errors
    ///
    /// Returns an error if allocating the transfer fails.
    pub fn new(
        dev_handle: Arc<DeviceHandle<C>>,
        request_type: u8,
        request: u8,
        value: u16,
        index: u16,
        data: &[u8],
    ) -> Result<Self> {
        let buffer = Vec::with_capacity(data.len() + LIBUSB_CONTROL_SETUP_SIZE);
        let kind = Control {
            request_type,
            request,
            value,
            index,
        };

        Transfer::alloc(dev_handle, 0, buffer, kind, 0)
    }

    /// Sets the transfer in the correct state to be reused. After
    /// calling this function, the transfer can be awaited again.
    ///
    /// # Errors
    ///
    /// Returns an error if replacing the transfer buffer fails.
    pub fn renew(
        &mut self,
        request_type: u8,
        request: u8,
        value: u16,
        index: u16,
        data: &[u8],
    ) -> Result<()> {
        let buffer = Vec::with_capacity(data.len() + LIBUSB_CONTROL_SETUP_SIZE);
        let kind = Control {
            request_type,
            request,
            value,
            index,
        };

        self.swap_buffer(buffer)?;
        self.kind = kind;
        self.state = TransferState::Allocated;
        Ok(())
    }
}

impl<C> FillTransfer for ControlTransfer<C>
where
    C: AsyncUsbContext,
{
    fn fill(&mut self, waker: Waker) -> Result<()> {
        let length = self.buffer.capacity() - LIBUSB_CONTROL_SETUP_SIZE;
        let length = length
            .try_into()
            .map_err(|_| Error::Other("Invalid buffer size"))?;

        let user_data = Box::into_raw(Box::new(TransferUserData::new(waker))).cast();

        unsafe {
            ffi::libusb_fill_control_setup(
                self.buffer.as_mut_ptr(),
                self.kind.request_type,
                self.kind.request,
                self.kind.value,
                self.kind.index,
                length,
            );

            ffi::libusb_fill_control_transfer(
                self.ptr.as_ptr(),
                self.dev_handle.as_raw(),
                self.buffer.as_mut_ptr(),
                Self::transfer_cb,
                user_data,
                0,
            );
        }

        Ok(())
    }
}

impl SingleBufferTransfer for Control {}

/// Raw control transfer kind.
#[allow(missing_copy_implementations)]
#[derive(Debug)]
pub struct RawControl(());

impl<C> RawControlTransfer<C>
where
    C: AsyncUsbContext,
{
    /// Constructs and allocates a new [`RawControlTransfer`].
    ///
    /// # Errors
    ///
    /// Returns an error if allocating the transfer fails.
    pub fn new(dev_handle: Arc<DeviceHandle<C>>, buffer: Vec<u8>) -> Result<Self> {
        Transfer::alloc(dev_handle, 0, buffer, RawControl(()), 0)
    }

    /// Sets the transfer in the correct state to be reused. After
    /// calling this function, the transfer can be awaited again.
    ///
    /// # Errors
    ///
    /// Returns an error if replacing the transfer buffer fails.
    pub fn renew(&mut self, buffer: Vec<u8>) -> Result<()> {
        self.swap_buffer(buffer)?;
        self.state = TransferState::Allocated;
        Ok(())
    }
}

impl<C> FillTransfer for RawControlTransfer<C>
where
    C: AsyncUsbContext,
{
    fn fill(&mut self, waker: Waker) -> Result<()> {
        let user_data = Box::into_raw(Box::new(TransferUserData::new(waker))).cast();

        unsafe {
            ffi::libusb_fill_control_transfer(
                self.ptr.as_ptr(),
                self.dev_handle.as_raw(),
                self.buffer.as_mut_ptr(),
                Self::transfer_cb,
                user_data,
                0,
            );
        }

        Ok(())
    }
}

impl SingleBufferTransfer for RawControl {}
