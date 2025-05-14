use std::{
    marker::PhantomData,
    os::fd::RawFd,
    ptr::{self, NonNull},
};

use crate::context::{AsyncUsbContext, EventHandler, EventHandlerData};
use rusb::ffi;

/// The kind of event that `libusb` reports that the file descriptor should be monitored for.
// NOTE: Should this be a full fledget bitmap of flags?
//       libusb seems to only care about POLLIN and POLLOUT, at least on Linux.
#[derive(Copy, Clone, Debug)]
pub enum FdEvents {
    Read,
    Write,
    ReadWrite, // Is this necessary?
    Other,
}

impl FdEvents {
    fn from_libusb(events: libc::c_short) -> Self {
        match events {
            x if x & (libc::POLLIN | libc::POLLOUT) != 0 => FdEvents::ReadWrite,
            x if x & libc::POLLIN != 0 => FdEvents::Read,
            x if x & libc::POLLOUT != 0 => FdEvents::Write,
            _ => FdEvents::Other,
        }
    }
}

/// Trait for setting callbacks that get called on file descriptor actions.
/// This should be used for constructing a [`FdCallbackRegistration`], which
/// implements [`EventHandler`].
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

    fd_callbacks.fd_added(context.clone(), fd, FdEvents::from_libusb(events));
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

/// Wrapper for implementing [`EventHandler`] for any
/// type that implements [`FdCallbacks`].
///
/// A generic `impl<T: FdCallbacks> EventHandler for T` would be better
/// but it would prevent downstream clients from coming up with their own
/// [`EventHandler`] implementations because the compiler will
/// complain that an implementation of `FdCallbacks` might be added by
/// an upstream crate.
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

impl<C, T> EventHandler<C> for FdCallbackRegistration<C, T>
where
    C: AsyncUsbContext,
    T: FdCallbacks<C>,
{
    fn setup(
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
                    let events = FdEvents::from_libusb(pollfd.as_ref().events);

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
    fn teardown(self: Box<Self>) {
        unsafe {
            ffi::libusb_set_pollfd_notifiers(self.context.as_raw(), None, None, ptr::null_mut());
        };
    }
}
