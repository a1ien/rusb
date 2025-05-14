use std::{
    marker::PhantomData,
    os::fd::RawFd,
    ptr::{self, NonNull},
};

use crate::context::{AsyncUsbContext, EventHandlerData, RegisterEventHandler};
use rusb::ffi;

#[derive(Copy, Clone, Debug)]
pub enum FdEvents {
    Read,
    Write,
    ReadWrite,
    Other,
}

impl FdEvents {
    fn from_libc(events: libc::c_short) -> Self {
        match events {
            x if x & (libc::POLLIN | libc::POLLOUT) != 0 => FdEvents::ReadWrite,
            x if x & libc::POLLIN != 0 => FdEvents::Read,
            x if x & libc::POLLOUT != 0 => FdEvents::Write,
            _ => FdEvents::Other,
        }
    }
}

#[derive(Debug)]
pub struct FdCallbackRegistration<C, T>
where
    C: AsyncUsbContext,
    T: FdCallbacks<C>,
{
    fd_callbacks: T,
    marker: PhantomData<fn() -> C>,
}

impl<C, T> FdCallbackRegistration<C, T>
where
    C: AsyncUsbContext,
    T: FdCallbacks<C>,
{
    pub fn new(fd_callbacks: T) -> Self {
        Self {
            fd_callbacks,
            marker: PhantomData,
        }
    }
}

/// Trait for setting callbacks that get called on file descriptor actions.
// NOTE: Should the fd methods get mutabable access to `self`?
//       It would make things easier, especially since event handling should
//       technically only be allowed by libusb in a single thread at a time.
pub trait FdCallbacks<C>: Send + Sync + 'static
where
    C: AsyncUsbContext,
{
    /// Gets called whenever a file descriptor is added.
    ///
    /// This method's job is to register the file descriptor with the runtime's
    /// event loop so that specific events get monitored and trigger non-blocking
    /// event handling with something like [`UsbContext::handle_events`].
    fn fd_added(&self, context: C, fd: RawFd, events: FdEvents);

    /// Gets called whenever a file descriptor is removed.
    ///
    /// This method's job is to essentially gracefully shut down the event
    /// monitoring that gets registered by [`FdCallbacks::fd_added`].
    fn fd_removed(&self, fd: RawFd);
}

/// The FFI wrapper callback over [`FdCallbacks::fd_added`].
extern "system" fn fd_added_cb<C, T>(
    fd: libc::c_int,
    events: libc::c_short,
    user_data: *mut libc::c_void,
) where
    C: AsyncUsbContext,
    T: FdCallbacks<C>,
{
    let CallbackUserdata::<C, T> {
        context,
        fd_callbacks,
    } = unsafe { &*user_data.cast() };

    fd_callbacks.fd_added(context.clone(), fd, FdEvents::from_libc(events));
}

/// The FFI wrapper callback over [`FdCallbacks::fd_removed`].
extern "system" fn fd_removed_cb<C, T>(fd: libc::c_int, user_data: *mut libc::c_void)
where
    C: AsyncUsbContext,
    T: FdCallbacks<C>,
{
    let CallbackUserdata::<C, T> { fd_callbacks, .. } = unsafe { &*user_data.cast() };
    fd_callbacks.fd_removed(fd);
}

#[derive(Debug)]
struct CallbackUserdata<C, T>
where
    C: AsyncUsbContext,
    T: FdCallbacks<C>,
{
    context: C,
    fd_callbacks: T,
}

impl<C, T> RegisterEventHandler<C> for FdCallbackRegistration<C, T>
where
    C: AsyncUsbContext,
    T: FdCallbacks<C>,
{
    fn register(
        self,
        context: C,
    ) -> crate::Result<Box<dyn crate::context::EventHandlerData<C> + 'static>> {
        let context_ptr = context.as_raw();

        unsafe {
            // Some file descriptors can get created when the context is initialized,
            // so get the file descriptors that the context already has and register
            // event sources for them through [`FdCallbacks::fd_added`].
            let pollfds_opt_ptr = NonNull::new(ffi::libusb_get_pollfds(context_ptr).cast_mut());

            if let Some(mut pollfds_ptr) = pollfds_opt_ptr {
                while let Some(pollfd) = NonNull::new(*pollfds_ptr.as_ptr()) {
                    let fd = pollfd.as_ref().fd;
                    let events = FdEvents::from_libc(pollfd.as_ref().events);

                    self.fd_callbacks.fd_added(context.clone(), fd, events);

                    pollfds_ptr = pollfds_ptr.add(1);
                }
            }

            let user_data_ptr = Box::into_raw(Box::new(CallbackUserdata {
                context,
                fd_callbacks: self.fd_callbacks,
            }));

            let handler_data = Box::from_raw(user_data_ptr);

            ffi::libusb_set_pollfd_notifiers(
                context_ptr,
                Some(fd_added_cb::<C, T>),
                Some(fd_removed_cb::<C, T>),
                user_data_ptr.cast(),
            );

            Ok(handler_data)
        }
    }
}

impl<C, T> EventHandlerData<C> for CallbackUserdata<C, T>
where
    C: AsyncUsbContext,
    T: FdCallbacks<C>,
{
    fn unregister(self: Box<Self>) {
        unsafe {
            ffi::libusb_set_pollfd_notifiers(self.context.as_raw(), None, None, ptr::null_mut());
        };
    }
}
