mod shared_state;
mod unhandled_transfer;

use self::{shared_state::OperationStatus, unhandled_transfer::UnhandledTransfer};
use std::marker::PhantomData;

use crate::{
    context::Context,
    device_handle::DeviceHandle,
    error::{Error, Result},
};

#[derive(Clone)]
pub enum TransferStatus {
    Completed,
    Error,
    Timeout,
    Cancelled,
    Stall,
    NoDevice,
    Overflow,
    Unknown,
}

impl TransferStatus {
    pub fn to_libusb_result(&self) -> Result<()> {
        match self {
            Self::Completed => Ok(()),
            Self::Timeout => Err(Error::Timeout),
            Self::Stall => Err(Error::Pipe),
            Self::NoDevice => Err(Error::NoDevice),
            Self::Overflow => Err(Error::Overflow),
            Self::Error | Self::Cancelled => Err(Error::Io),
            _ => Err(Error::Other),
        }
    }
}

impl TransferStatus {
    fn from_libusb(code: libc::c_int) -> Self {
        match code {
            0 => TransferStatus::Completed,
            1 => TransferStatus::Error,
            2 => TransferStatus::Timeout,
            3 => TransferStatus::Cancelled,
            4 => TransferStatus::Stall,
            5 => TransferStatus::NoDevice,
            6 => TransferStatus::Overflow,
            _ => TransferStatus::Unknown,
        }
    }
}

pub type TransferCallbackFunction = Option<Box<dyn FnMut(TransferStatus, Vec<u8>)>>;

pub struct Transfer<'a> {
    unhandled_transfer: *mut UnhandledTransfer,
    _device_handle: PhantomData<&'a DeviceHandle<Context>>,
}

impl<'a> Drop for Transfer<'a> {
    fn drop(&mut self) {
        let destroy_unhandled_transfer = {
            let mut state = unsafe { (*self.unhandled_transfer).state.lock().unwrap() };
            match state.status {
                OperationStatus::Busy => {
                    state.is_transfer_dropped = true;
                    false
                }
                OperationStatus::Completed => true,
            }
        };

        if destroy_unhandled_transfer {
            unsafe { Box::from_raw(self.unhandled_transfer) };
        }
    }
}

impl<'a> Transfer<'a> {
    pub fn new(device_handle: &'a mut DeviceHandle<Context>, iso_packets: i32) -> Result<Self> {
        let unhandled_transfer = UnhandledTransfer::new(device_handle, iso_packets)?;
        let transfer = Self {
            unhandled_transfer,
            _device_handle: PhantomData,
        };

        Ok(transfer)
    }

    pub fn set_endpoint(&mut self, endpoint: u8) -> Result<()> {
        unsafe {
            (*(*self.unhandled_transfer).handle).endpoint = endpoint;
        };
        Ok(())
    }

    pub fn set_transfer_type(&mut self, transfer_type: u8) -> Result<()> {
        unsafe {
            (*(*self.unhandled_transfer).handle).transfer_type = transfer_type;
        };
        Ok(())
    }

    pub fn set_status(&mut self, status: i32) -> Result<()> {
        unsafe {
            (*(*self.unhandled_transfer).handle).status = status;
        };
        Ok(())
    }

    pub fn set_timeout(&mut self, timeout: u32) -> Result<()> {
        unsafe {
            (*(*self.unhandled_transfer).handle).timeout = timeout;
        };
        Ok(())
    }

    pub fn set_callback(&mut self, callback: TransferCallbackFunction) -> Result<()> {
        let mut state = unsafe { (*self.unhandled_transfer).state.lock().unwrap() };
        state.callback = callback;
        Ok(())
    }

    pub fn submit_transfer(
        &mut self,
        data: &[u8],
        bm_request_type: u8,
        b_request: u8,
        w_value: u16,
        w_index: u16,
    ) -> Result<()> {
        let mut unhandled_transfer = unsafe { Box::from_raw(self.unhandled_transfer) };
        let result =
            unhandled_transfer.submit_transfer(data, bm_request_type, b_request, w_value, w_index);

        Box::into_raw(unhandled_transfer);
        result?;

        Ok(())
    }

    pub fn cancel_transfer(&mut self) -> Result<()> {
        let mut unhandled_transfer = unsafe { Box::from_raw(self.unhandled_transfer) };
        let result = unhandled_transfer.cancel_transfer();

        Box::into_raw(unhandled_transfer);
        result?;

        Ok(())
    }
}
