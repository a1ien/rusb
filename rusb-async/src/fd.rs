//! Provides utilites for integrating file descriptor monitoring into runtime specific event loops.
//! This is only availble on UNIX-like systems.

use std::{
    os::fd::RawFd,
    ptr::{self, NonNull},
};

use rusb::{ffi, UsbContext};

/// Trait for setting callbacks that get called on file descriptor actions.
// NOTE: Should the fd methods get mutabable access to `self`?
//       It would make things easier, especially since event handling should
//       technically only be allowed by libusb in a single thread at a time.
pub trait FdCallbacks {
    type Context: UsbContext;

    /// Return a reference to the context the callbacks apply to.
    fn context(&self) -> &Self::Context;

    /// Gets called whenever a file descriptor is added.
    ///
    /// This method's job is to register the file descriptor with the runtime's
    /// event loop so that specific events like [`libc::POLLIN`] or [`libc::POLLOUT`]
    /// get monitored and trigger non-blocking event handling with something like [`UsbContext::handle_events`].
    fn fd_added(&self, fd: RawFd, events: libc::c_short);

    /// Gets called whenever a file descriptor is removed.
    ///
    /// This method's job is to essentially gracefully shut down the event
    /// monitoring that gets registered by [`FdCallbacks::fd_added`].
    fn fd_removed(&self, fd: RawFd);
}

/// The file descriptor event handler takes care of initializing the callbacks set
/// through [`FdCallbacks`].
///
/// The callbacks get set on construction and will get de-registered on drop.
/// Therefore, for correct [`std::future::Future`] handling, the [`FdEventHandler`]
/// instance associated with the given [`UsbContext`] type must live as long as the context.
///
/// You should not have multiple [`FdEventHandler`] instances at the same time for the same
/// [`UsbContext`] implementing type instance. Dropping an old [`FdEventHandler`] after a new
/// one gets constructed (and overwrites the file descriptor callbacks) will result in the
/// callbacks getting unset, even if the second [`FdEventHandler`] instance is still live.
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
            // Some file descriptors can get created when the context is initialized,
            // so get the file descriptors that the context already has and register
            // event sources for them through [`FdCallbacks::fd_added`].
            let pollfds_opt_ptr = NonNull::new(ffi::libusb_get_pollfds(context).cast_mut());
            if let Some(mut pollfds_ptr) = pollfds_opt_ptr {
                while let Some(pollfd) = NonNull::new(*pollfds_ptr.as_ptr()) {
                    let fd = pollfd.as_ref().fd;
                    let events = pollfd.as_ref().events;

                    fd_callbacks.fd_added(fd, events);
                    pollfds_ptr = pollfds_ptr.add(1);
                }
            }

            // We'll only ever access a reference of this from now on until
            // this [`FdEventHandler`] gets dropped.
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

    /// The FFI wrapper callback over [`FdCallbacks::fd_added`].
    extern "system" fn fd_added_cb(
        fd: libc::c_int,
        events: libc::c_short,
        user_data: *mut libc::c_void,
    ) {
        unsafe { &*user_data.cast::<T>() }.fd_added(fd, events);
    }

    /// The FFI wrapper callback over [`FdCallbacks::fd_removed`].
    extern "system" fn fd_removed_cb(fd: libc::c_int, user_data: *mut libc::c_void) {
        unsafe { &*user_data.cast::<T>() }.fd_removed(fd);
    }
}

/// SAFETY: This is safe since the FdEventHandler inner pointer is boxed.
unsafe impl<C, T> Send for FdEventHandler<C, T>
where
    C: UsbContext,
    T: FdCallbacks<Context = C> + Send,
{
}

/// SAFETY: This is safe since the FdEventHandler inner pointer is only
/// accessed immutably, except when dropped.
unsafe impl<C, T> Sync for FdEventHandler<C, T>
where
    C: UsbContext,
    T: FdCallbacks<Context = C> + Sync,
{
}

/// Deregisters the file descriptor handling callbacks and drops the [`FdCallbacks`] inner pointer.
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
