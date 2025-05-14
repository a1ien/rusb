#[cfg(unix)]
mod fd_callbacks;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex, OnceLock},
};

#[cfg(unix)]
pub use fd_callbacks::{FdCallbackRegistration, FdCallbacks, FdEvents};
use rusb::{ffi::libusb_context, Context, GlobalContext, UsbContext, UsbOption};

use crate::Error;

/// A `libusb` context meant for async transfers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AsyncContext {
    context: Arc<AsyncContextInner>,
}

/// TODO: This ends up being double Arc'ed :(.
///
///       Probably the best way to get rid of this is
///       by integrating `rust-async` into `rusb`, thus
///       having access to all the internals.
///
///       Another option would be having a common crate
///       to construct something of a [`ContextInner`],
///       but that would have to be published as well.
///
///       Alternatively, some code copying code be done
///       to replicate the [`ContextInner`] type, but
///       that also implies `rusb::Error::from_libusb`
///       and other stuff.
#[derive(Debug, Eq, PartialEq)]
struct AsyncContextInner(Context);

impl Drop for AsyncContextInner {
    fn drop(&mut self) {
        let ctx = self.0.as_raw();

        if let Some(map) = EVENT_DATA_HANDLER_MAP.get() {
            if let Some(handler_data) = map.inner.lock().unwrap().remove(&ctx) {
                handler_data.teardown();
            }
        }
    }
}

type BoxEventHandlerData = Box<dyn EventHandlerData<AsyncContext>>;

#[derive(Default)]
struct EventHandlerDataMap {
    inner: Mutex<HashMap<*mut libusb_context, BoxEventHandlerData>>,
}

unsafe impl Sync for EventHandlerDataMap {}
unsafe impl Send for EventHandlerDataMap {}

static EVENT_DATA_HANDLER_MAP: OnceLock<EventHandlerDataMap> = OnceLock::new();

/// Trait implemented by contexts that can be used for async transfers.
///
/// It is recommended to use an [`AsyncContext`], although the trait is
/// implemented for [`GlobalContext`] as well. However, the [`GlobalContext`]
/// needs explicit event handler registration using [`AsyncUsbContext::set_event_handler`]
/// or manual polling.
pub trait AsyncUsbContext: UsbContext + 'static {
    /// Sets an event handler for a given context.
    ///
    /// [`AsyncContext`] instances get created with an event handler, and this
    /// method allows swapping it.
    ///
    /// The same applies for [`GlobalContext`], except that it **DOES NOT** start
    /// with an event handler, therefore one must be set using this method. Failing
    /// to do so will lead to async transfers never finishing, unless some other kind
    /// of manual polling is performed.
    fn set_event_handler<T>(&self, register_event_handler: T) -> crate::Result<()>
    where
        T: EventHandler<Self>;
}

/// Used to initialize custom event handling for a context.
///
/// The event handler can be a [`Future`], a background thread or, for UNIX-like systems,
/// an event loop integration of file descriptor monitoring through [`FdCallbacks`].
pub trait EventHandler<C>
where
    C: AsyncUsbContext,
{
    /// Starts the event handling.
    ///
    /// Returns a boxed trait object that implements [`EventHandlerData`], which is
    /// used for stopping the event handler. This allows swapping the event handler
    /// for a context throughout its lifetime and cleanup of resources.
    fn setup(self, context: C) -> crate::Result<Box<dyn EventHandlerData<C>>>;
}

/// Data returned from initializing an event handler.
pub trait EventHandlerData<C>: Send + Sync + 'static
where
    C: AsyncUsbContext,
{
    /// Stops the event handler that returned this type, allowing another one to take its place.
    // NOTE: Should this be fallible?
    fn teardown(self: Box<Self>);
}

impl UsbContext for AsyncContext {
    fn as_raw(&self) -> *mut libusb_context {
        self.context.0.as_raw()
    }
}

impl AsyncUsbContext for AsyncContext {
    fn set_event_handler<T>(&self, register_event_handler: T) -> crate::Result<()>
    where
        T: EventHandler<Self>,
    {
        let ctx_ptr = self.as_raw();

        let mut map = EVENT_DATA_HANDLER_MAP
            .get_or_init(EventHandlerDataMap::default)
            .inner
            .lock()
            .unwrap();

        if let Some(handler_data) = map.remove(&ctx_ptr) {
            handler_data.teardown();
        }

        let handler_data = register_event_handler.setup(self.clone())?;
        map.insert(ctx_ptr, handler_data);

        Ok(())
    }
}

impl AsyncUsbContext for GlobalContext {
    fn set_event_handler<T>(&self, register_event_handler: T) -> crate::Result<()>
    where
        T: EventHandler<Self>,
    {
        static EVENT_HANDLER: OnceLock<Option<Box<dyn EventHandlerData<GlobalContext>>>> =
            OnceLock::new();

        if let Err(Some(handler_data)) = EVENT_HANDLER.set(None) {
            handler_data.teardown();
        }

        let handler_data = register_event_handler.setup(GlobalContext::default())?;
        EVENT_HANDLER.set(Some(handler_data)).ok();
        Ok(())
    }
}

impl AsyncContext {
    pub fn new<T>(register_event_handler: T) -> crate::Result<Self>
    where
        T: EventHandler<Self>,
    {
        let ctx = Context::new().map_err(|_| Error::Other("Context creation failed"))?;
        let context = Arc::new(AsyncContextInner(ctx));
        let this = Self { context };
        let handler_data = register_event_handler.setup(this.clone())?;

        EVENT_DATA_HANDLER_MAP
            .get_or_init(EventHandlerDataMap::default)
            .inner
            .lock()
            .unwrap()
            .insert(this.as_raw(), handler_data);

        Ok(this)
    }

    /// Creates a new `libusb` context and sets runtime options.
    pub fn with_options<T>(register_event_handler: T, opts: &[UsbOption]) -> crate::Result<Self>
    where
        T: EventHandler<Self>,
    {
        let ctx =
            Context::with_options(opts).map_err(|_| Error::Other("Context creation failed"))?;
        let context = Arc::new(AsyncContextInner(ctx));
        let this = Self { context };
        let handler_data = register_event_handler.setup(this.clone())?;

        EVENT_DATA_HANDLER_MAP
            .get_or_init(EventHandlerDataMap::default)
            .inner
            .lock()
            .unwrap()
            .insert(this.as_raw(), handler_data);

        Ok(this)
    }
}
