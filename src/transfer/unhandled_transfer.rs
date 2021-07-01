mod async_awakener;

use self::async_awakener::AsyncAwakener;

use libusb1_sys::constants::*;
use libusb1_sys::*;

use std::sync::{Arc, Mutex};

use super::{
    shared_state::{SharedState, SharedStatePtr},
    OperationStatus, TransferStatus,
};
use crate::{
    context::Context,
    device_handle::DeviceHandle,
    error::{Error, Result},
    UsbContext,
};

pub struct UnhandledTransfer {
    pub handle: *mut libusb_transfer,
    pub state: SharedStatePtr,
    _awakener: AsyncAwakener,
    data: Vec<u8>,
}

impl UnhandledTransfer {
    pub fn new(device_handle: &mut DeviceHandle<Context>, iso_packets: i32) -> Result<*mut Self> {
        let handle = Self::allocate_transfer_handle(iso_packets)?;

        unsafe {
            (*handle).dev_handle = device_handle.as_raw();
            (*handle).callback = libusb_transfer_callback_function;
        }

        let local_context = device_handle.context().clone();
        let awakener = AsyncAwakener::spawn(move || unsafe {
            libusb_handle_events_completed(local_context.as_raw(), std::ptr::null_mut());
        });

        let transfer = Box::new(Self {
            handle,
            state: Arc::new(Mutex::new(SharedState::new(handle))),
            _awakener: awakener,
            data: vec![],
        });
        let transfer = Box::into_raw(transfer);

        unsafe {
            (*handle).user_data =
                std::mem::transmute::<*mut UnhandledTransfer, *mut libc::c_void>(transfer);
        }

        Ok(transfer)
    }

    pub fn drop(handle: *mut libusb_transfer) {
        unsafe {
            libusb_cancel_transfer(handle);
            libusb_free_transfer(handle);
        }
    }

    pub fn submit_transfer(
        &mut self,
        data: &[u8],
        bm_request_type: u8,
        b_request: u8,
        w_value: u16,
        w_index: u16,
    ) -> Result<()> {
        let mut state = match self.state.lock() {
            Err(_) => return Err(Error::Other),
            Ok(state) => state,
        };

        if let OperationStatus::Busy = state.status {
            return Err(Error::Busy);
        }
        state.status = OperationStatus::Busy;

        let setup_packet = Self::create_setup_packet(
            bm_request_type,
            b_request,
            w_value,
            w_index,
            data.len() as u16,
        )?;

        self.data = setup_packet
            .into_iter()
            .chain(data.iter().copied())
            .collect::<Vec<u8>>();

        unsafe {
            (*self.handle).buffer = self.data.as_mut_ptr();
            (*self.handle).length = self.data.len() as i32;
        }

        try_unsafe!(libusb_submit_transfer(self.handle));

        Ok(())
    }

    pub fn cancel_transfer(&mut self) -> Result<()> {
        try_unsafe!(libusb_cancel_transfer(self.handle));
        Ok(())
    }

    fn allocate_transfer_handle(iso_packets: i32) -> Result<*mut libusb_transfer> {
        let transfer_handle = unsafe { libusb_alloc_transfer(iso_packets) };
        if transfer_handle == std::ptr::null_mut() {
            return Err(Error::NoMem);
        }

        Ok(transfer_handle)
    }

    fn create_setup_packet(
        bm_request_type: u8,
        b_request: u8,
        w_value: u16,
        w_index: u16,
        data_size: u16,
    ) -> Result<Vec<u8>> {
        // Actual at the time of version 1.0.23
        static_assertions::const_assert!(LIBUSB_CONTROL_SETUP_SIZE == 8);

        let mut setup_packet = vec![bm_request_type, b_request];
        setup_packet.extend(w_value.to_le_bytes().iter());
        setup_packet.extend(w_index.to_le_bytes().iter());
        setup_packet.extend(data_size.to_le_bytes().iter());

        Ok(setup_packet)
    }
}

extern "system" fn libusb_transfer_callback_function(transfer_handle: *mut libusb_transfer) {
    let transfer = unsafe {
        Box::from_raw(std::mem::transmute::<
            *mut libc::c_void,
            *mut UnhandledTransfer,
        >((*transfer_handle).user_data))
    };

    let state = transfer.state.clone();
    let mut state = state.lock().unwrap();

    if state.is_transfer_dropped {
        UnhandledTransfer::drop(state.handle);
    } else {
        state.status = OperationStatus::Completed;

        let status = unsafe { (*state.handle).status };
        if let Some(ref mut callback) = state.callback {
            let status = TransferStatus::from_libusb(status);
            let actual_length = unsafe { (*transfer_handle).actual_length };
            callback(
                status,
                transfer
                    .data
                    .iter()
                    .copied()
                    .take(actual_length as usize)
                    .collect(),
            );
        }

        // forget about transfer
        Box::into_raw(transfer);
    }
}
