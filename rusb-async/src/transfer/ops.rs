use crate::AsyncUsbContext;

use crate::{error::Result, transfer::Transfer};
use std::task::Waker;

pub trait FillTransfer {
    /// Step 2 of the async API.
    ///
    /// Fills the transfer in preparation for submitting it. Apart
    /// from transfer specific setup, this method will also pass
    /// the [`Waker`] obtained when the transfer gets polled first.
    ///
    /// This way, when the transfer completes, the waker notifies the runtime
    /// that the transfer can be polled so it can complete.
    ///
    /// # Errors
    ///
    /// Returns an error if filling the transfer fails.
    fn fill(&mut self, waker: Waker) -> Result<()>;
}

/// Abstraction over transfer completion.
///
/// This is mainly to acommodate isochronous transfers, since their
/// output is not a single buffer.
pub trait CompleteTransfer: FillTransfer {
    type Output;

    /// Consume the transfer buffer to provide the given output.
    /// For non-isochronous transfers this will simply be the buffer itself.
    ///
    /// # Errors
    ///
    /// Returns an error if consuming the buffer fails.
    fn consume_buffer(&mut self, buffer: Vec<u8>) -> Result<Self::Output>;
}

/// Marker trait for common implementation of [`CompleteTransfer`] for
/// non-isochronous endpoints.
pub trait SingleBufferTransfer {}

/// Implementation for essentially all non-isochronous transfers. The
/// transfer output will be the data buffer itself.
impl<C, K> CompleteTransfer for Transfer<C, K>
where
    C: AsyncUsbContext,
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
