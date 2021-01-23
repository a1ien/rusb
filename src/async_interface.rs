
use std::collections::VecDeque;
use std::pin::Pin;

use libc::c_void;
use libusb1_sys::libusb_transfer;

use crate::Context;
use crate::DeviceHandle;

#[derive(Debug)]
pub enum TransferStatus {
	Completed(Vec<u8>),
	Error,
	TimedOut,
	Stall,
	NoDevice,
	Overflow,
	Unknown(i32)
}

// Putting the libusb_transfer pointer and the buffer in the struct that owns both ties the lifetimes
// together and ensures that `buff` will live as long as `ptr`.  We also know that the receiving VecDeque
// will live as long as the pointer and buffer because it's owned by the same struct.  We put the pointer
// inside `recv` to the `libusb_transfer` struct, so it's important for `recv` to stay at the same address,
// which is why it's wrapped in a Pin<_>
// This is not thread-safe at all.  If you try to implement Send or Sync on it, it'll have to be an unsafe impl.
pub struct AsyncTransfer {
	pub ptr: *mut libusb_transfer,
	pub buff: Vec<u8>,
	pub recv: Pin<Box<VecDeque<TransferStatus>>>,
}

impl AsyncTransfer {

	extern "system" fn callback(xfer_ptr: *mut libusb_transfer) {

		unsafe {
			let xfer = match (*xfer_ptr).status {
				0 => {
					// Clone the memory into a vector
					let slice:&[u8] = std::slice::from_raw_parts((*xfer_ptr).buffer, (*xfer_ptr).actual_length as usize);
					TransferStatus::Completed(slice.to_vec())
				},
				1 => TransferStatus::Error,
				2 => TransferStatus::TimedOut,
				3 => TransferStatus::Stall,
				4 => TransferStatus::NoDevice,
				5 => TransferStatus::Overflow,
				n => TransferStatus::Unknown(n),
			};

			// Update the parser stored in the user data field
			let xfer_deque:*mut VecDeque<TransferStatus> = (*xfer_ptr).user_data as *mut VecDeque<TransferStatus>;
			(*xfer_deque).push_back(xfer);

			// Resubmit the transfer
			assert!(libusb1_sys::libusb_submit_transfer(xfer_ptr) == 0);
		}

	}

	pub fn bulk(handle: &mut DeviceHandle<Context>, addr:u8) -> Self {

		// The steps in the comments follow the steps described in the libusb documentation

		// Step 1: Allocation		
		let ptr:*mut libusb_transfer = unsafe{ libusb1_sys::libusb_alloc_transfer(0) };
		let mut buff:Vec<u8> = vec![0u8; 256];
		assert!(!ptr.is_null());

		let mut user_data = Box::new(VecDeque::new());

		// Step 2: Filling
		let default_timeout_ms = 1000;
		unsafe {
			let rp_ptr:&mut VecDeque<TransferStatus> = &mut user_data;
			libusb1_sys::libusb_fill_bulk_transfer(ptr, handle.as_raw(), addr,
				buff.as_mut_ptr(), buff.len() as i32, Self::callback, 
				rp_ptr as *mut VecDeque<TransferStatus> as *mut c_void, default_timeout_ms);
		}

		// Step 3: Submission
		unsafe {
			assert!(libusb1_sys::libusb_submit_transfer(ptr) == 0);
		}

		// Pin protects the location in memory by making possible to access the data in the
		// pointer type, but not possible to get the actual location in memory.  You can't
		// move it if you don't know where it is.  We got the location in memory before we
		// wrapped this pointer in a Pin<_> so that we could populate the bulk transfer.  Now
		// we know it's going to stay there until the memory gets freed.
		let recv:Pin<Box<VecDeque<TransferStatus>>> = Pin::new(user_data);

		Self{ ptr, buff, recv }
	}

}

impl std::ops::Drop for AsyncTransfer {

	fn drop(&mut self) {
		unsafe{ libusb1_sys::libusb_free_transfer(self.ptr); }
	}

}



