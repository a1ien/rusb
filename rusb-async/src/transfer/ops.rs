use rusb::UsbContext;

use crate::{error::Result, transfer::Transfer};
use std::task::Waker;

/// Step 2 of the async API.
pub trait FillTransfer {
    /// # Errors
    fn fill(&mut self, waker: Waker) -> Result<()>;
}

pub trait CompleteTransfer: FillTransfer {
    type Output;

    /// # Errors
    fn consume_buffer(&mut self, buffer: Vec<u8>) -> Result<Self::Output>;
}

/// Marker trait for common implementation of [`CompleteTransfer`] for
/// non-isochronous endpoints.
pub trait SingleBufferTransfer {}

/// Implementation for essentially all non-isochronous transfers.
impl<C, K> CompleteTransfer for Transfer<C, K>
where
    C: UsbContext,
    K: SingleBufferTransfer + Unpin,
    Self: FillTransfer,
{
    type Output = Vec<u8>;

    fn consume_buffer(&mut self, mut buffer: Vec<u8>) -> Result<Self::Output> {
        let len = self.transfer().actual_length.try_into().unwrap();
        unsafe { buffer.set_len(len) };
        Ok(buffer)
    }
}
