use super::TransferCallbackFunction;
use libusb1_sys::*;
use std::sync::{Arc, Mutex};

pub enum OperationStatus {
    Busy,
    Completed,
}

pub struct SharedState {
    pub handle: *mut libusb_transfer,
    pub callback: TransferCallbackFunction,
    pub status: OperationStatus,
    pub is_transfer_dropped: bool,
}
pub type SharedStatePtr = Arc<Mutex<SharedState>>;

impl SharedState {
    pub fn new(handle: *mut libusb_transfer) -> Self {
        Self {
            handle,
            callback: None,
            status: OperationStatus::Completed,
            is_transfer_dropped: false,
        }
    }
}
