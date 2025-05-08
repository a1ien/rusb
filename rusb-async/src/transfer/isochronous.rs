use std::{slice, sync::Arc, task::Waker};

use rusb::{
    constants::{LIBUSB_ENDPOINT_DIR_MASK, LIBUSB_ENDPOINT_OUT, LIBUSB_TRANSFER_COMPLETED},
    ffi::{self, libusb_iso_packet_descriptor},
    DeviceHandle, UsbContext,
};

use crate::{
    error::{Error, Result},
    transfer::{CompleteTransfer, FillTransfer, Transfer, TransferState},
};

pub type IsochronousTransfer<C> = Transfer<C, Isochronous>;

#[allow(missing_copy_implementations)]
#[derive(Debug)]
pub struct Isochronous {
    iso_packets: i32,
}

impl<C> IsochronousTransfer<C>
where
    C: UsbContext,
{
    /// # Errors
    pub fn new(
        dev_handle: Arc<DeviceHandle<C>>,
        endpoint: u8,
        buffer: Vec<u8>,
        iso_packets: i32,
    ) -> Result<Self> {
        Self::alloc(
            dev_handle,
            endpoint,
            buffer,
            Isochronous { iso_packets },
            iso_packets,
        )
    }

    /// # Errors
    pub fn reuse(&mut self, endpoint: u8, buffer: Vec<u8>) -> Result<()> {
        self.endpoint = endpoint;
        self.swap_buffer(buffer)?;
        self.state = TransferState::Allocated;
        Ok(())
    }

    fn packet_descriptors(&self) -> &[libusb_iso_packet_descriptor] {
        unsafe {
            slice::from_raw_parts(
                self.transfer().iso_packet_desc.as_ptr(),
                self.transfer().num_iso_packets.try_into().unwrap(),
            )
        }
    }
}

impl<C> FillTransfer for IsochronousTransfer<C>
where
    C: UsbContext,
{
    fn fill(&mut self, waker: Waker) -> Result<()> {
        let length = if self.endpoint & LIBUSB_ENDPOINT_DIR_MASK == LIBUSB_ENDPOINT_OUT {
            // for OUT endpoints: the currently valid data in the buffer
            self.buffer.len()
        } else {
            // for IN endpoints: the full capacity
            self.buffer.capacity()
        };

        let length: i32 = length
            .try_into()
            .map_err(|_| Error::Other("Invalid buffer length"))?;

        let packet_lengths = (length / self.kind.iso_packets)
            .try_into()
            .map_err(|_| Error::Other("Invalid iso packets length"))?;

        let user_data = Box::into_raw(Box::new(waker)).cast();

        unsafe {
            ffi::libusb_fill_iso_transfer(
                self.ptr.as_ptr(),
                self.dev_handle.as_raw(),
                self.endpoint,
                self.buffer.as_mut_ptr(),
                length,
                self.kind.iso_packets,
                Self::transfer_cb,
                user_data,
                0,
            );

            ffi::libusb_set_iso_packet_lengths(self.ptr.as_ptr(), packet_lengths);
        }

        Ok(())
    }
}

impl<C> CompleteTransfer for IsochronousTransfer<C>
where
    C: UsbContext,
{
    type Output = IsochronousBuffer;

    fn consume_buffer(&mut self, mut buffer: Vec<u8>) -> Result<Self::Output> {
        debug_assert!(self.transfer().length >= self.transfer().actual_length);
        let len = self.transfer().length.try_into().unwrap();
        unsafe { buffer.set_len(len) };

        let packet_descriptors = self
            .packet_descriptors()
            .iter()
            .map(TryFrom::try_from)
            .collect::<Result<_>>()?;

        Ok(IsochronousBuffer {
            packet_descriptors,
            buffer,
        })
    }
}

#[derive(Debug, Copy, Clone)]
struct IsochronousPacketDescriptor {
    length: usize,
    actual_length: usize,
    status: libc::c_int,
}

impl TryFrom<&libusb_iso_packet_descriptor> for IsochronousPacketDescriptor {
    type Error = Error;

    fn try_from(value: &libusb_iso_packet_descriptor) -> std::result::Result<Self, Self::Error> {
        let length = value
            .length
            .try_into()
            .map_err(|_| Error::Other("Invalid isochronous packet length"))?;
        let actual_length = value
            .length
            .try_into()
            .map_err(|_| Error::Other("Invalid isochronous packet actual length"))?;

        Ok(Self {
            length,
            actual_length,
            status: value.status,
        })
    }
}

#[derive(Clone, Debug)]
pub struct IsochronousBuffer {
    packet_descriptors: Vec<IsochronousPacketDescriptor>,
    buffer: Vec<u8>,
}

impl IsochronousBuffer {
    #[must_use]
    pub fn iter(&self) -> IsoBufIter<'_> {
        self.into_iter()
    }
}

impl<'a> IntoIterator for &'a IsochronousBuffer {
    type Item = &'a [u8];

    type IntoIter = IsoBufIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        IsoBufIter {
            packet_descriptors_iter: self.packet_descriptors.iter(),
            buffer: &self.buffer,
            offset: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct IsoBufIter<'a> {
    packet_descriptors_iter: slice::Iter<'a, IsochronousPacketDescriptor>,
    buffer: &'a [u8],
    offset: usize,
}

impl<'a> Iterator for IsoBufIter<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let packet_desc = self.packet_descriptors_iter.next()?;
            let packet_start = self.offset;
            self.offset += packet_desc.length;

            if packet_desc.status == LIBUSB_TRANSFER_COMPLETED {
                let packet_end = packet_start + packet_desc.actual_length;
                return Some(&self.buffer[packet_start..packet_end]);
            }
        }
    }
}
