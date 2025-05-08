use std::{
    os::fd::RawFd,
    ptr::{self, NonNull},
};

use rusb::{ffi, UsbContext};

pub trait FdCallbacks {
    type Context: UsbContext;

    fn context(&self) -> &Self::Context;

    fn fd_added(&self, fd: RawFd, events: libc::c_short);

    fn fd_removed(&self, fd: RawFd);
}

#[derive(Debug)]
pub struct FdEventHandler<C, M>(*mut M)
where
    C: UsbContext,
    M: FdCallbacks<Context = C>;

impl<C, M> FdEventHandler<C, M>
where
    C: UsbContext,
    M: FdCallbacks<Context = C>,
{
    pub fn new(fd_monitor: M) -> Self {
        let context = fd_monitor.context().as_raw();

        unsafe {
            let pollfds_opt_ptr = NonNull::new(ffi::libusb_get_pollfds(context).cast_mut());
            if let Some(mut pollfds_ptr) = pollfds_opt_ptr {
                while let Some(pollfd) = NonNull::new(*pollfds_ptr.as_ptr()) {
                    let fd = pollfd.as_ref().fd;
                    let events = pollfd.as_ref().events;

                    fd_monitor.fd_added(fd, events);
                    pollfds_ptr = pollfds_ptr.add(1);
                }
            }

            let fd_monitor_ptr = Box::into_raw(Box::new(fd_monitor));
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
        unsafe { &*user_data.cast::<M>() }.fd_added(fd, events);
    }

    extern "system" fn fd_removed_cb(fd: libc::c_int, user_data: *mut libc::c_void) {
        unsafe { &*user_data.cast::<M>() }.fd_removed(fd);
    }
}

unsafe impl<C, M> Send for FdEventHandler<C, M>
where
    C: UsbContext,
    M: FdCallbacks<Context = C> + Send,
{
}

unsafe impl<C, M> Sync for FdEventHandler<C, M>
where
    C: UsbContext,
    M: FdCallbacks<Context = C> + Sync,
{
}

impl<C, M> Drop for FdEventHandler<C, M>
where
    C: UsbContext,
    M: FdCallbacks<Context = C>,
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
