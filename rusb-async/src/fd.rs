use std::{
    os::fd::RawFd,
    ptr::{self, NonNull},
};

use rusb::{ffi, UsbContext};

// NOTE: Should the fd methods get mutabable access to `self`?
//       It would make things easier, especially since polling should technically only
//       be allowed by libusb in a single thread at a time.
pub trait FdCallbacks {
    type Context: UsbContext;

    fn context(&self) -> &Self::Context;

    fn fd_added(&self, fd: RawFd, events: libc::c_short);

    fn fd_removed(&self, fd: RawFd);
}

#[derive(Debug)]
pub struct FdEventHandler<C, T>(*mut T)
where
    C: UsbContext,
    T: FdCallbacks<Context = C>;

impl<C, T> FdEventHandler<C, T>
where
    C: UsbContext,
    T: FdCallbacks<Context = C>,
{
    pub fn new(fd_callbacks: T) -> Self {
        let context = fd_callbacks.context().as_raw();

        unsafe {
            let pollfds_opt_ptr = NonNull::new(ffi::libusb_get_pollfds(context).cast_mut());
            if let Some(mut pollfds_ptr) = pollfds_opt_ptr {
                while let Some(pollfd) = NonNull::new(*pollfds_ptr.as_ptr()) {
                    let fd = pollfd.as_ref().fd;
                    let events = pollfd.as_ref().events;

                    fd_callbacks.fd_added(fd, events);
                    pollfds_ptr = pollfds_ptr.add(1);
                }
            }

            let fd_monitor_ptr = Box::into_raw(Box::new(fd_callbacks));
            let user_data = fd_monitor_ptr.cast();

            ffi::libusb_set_pollfd_notifiers(
                context,
                Some(Self::fd_added_cb),
                Some(Self::fd_removed_cb),
                user_data,
            );

            Self(fd_monitor_ptr)
        }
    }

    extern "system" fn fd_added_cb(fd: libc::c_int, events: libc::c_short, user_data: *mut libc::c_void) {
        unsafe { &*user_data.cast::<T>() }.fd_added(fd, events);
    }

    extern "system" fn fd_removed_cb(fd: libc::c_int, user_data: *mut libc::c_void) {
        unsafe { &*user_data.cast::<T>() }.fd_removed(fd);
    }
}

unsafe impl<C, T> Send for FdEventHandler<C, T>
where
    C: UsbContext,
    T: FdCallbacks<Context = C> + Send,
{
}

unsafe impl<C, T> Sync for FdEventHandler<C, T>
where
    C: UsbContext,
    T: FdCallbacks<Context = C> + Sync,
{
}

impl<C, T> Drop for FdEventHandler<C, T>
where
    C: UsbContext,
    T: FdCallbacks<Context = C>,
{
    fn drop(&mut self) {
        unsafe {
            let fd_monitor = Box::from_raw(self.0);

            ffi::libusb_set_pollfd_notifiers(
                fd_monitor.context().as_raw(),
                None,
                None,
                ptr::null_mut(),
            );
        }
    }
}
