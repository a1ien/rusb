#[cfg(unix)]
mod fd_callbacks;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex, OnceLock},
};

pub use fd_callbacks::{FdCallbackRegistration, FdCallbacks, FdEvents};
use rusb::{ffi::libusb_context, Context, GlobalContext, UsbContext, UsbOption};

use crate::Error;

/// A `libusb` context meant for async transfers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AsyncContext {
    context: Arc<AsyncContextInner>,
}

#[derive(Debug, Eq, PartialEq)]
struct AsyncContextInner(Context);

impl Drop for AsyncContextInner {
    fn drop(&mut self) {
        let ctx = self.0.as_raw();

        if let Some(map) = EVENT_DATA_HANDLER_MAP.get() {
            if let Some(handler_data) = map.inner.lock().unwrap().remove(&ctx) {
                handler_data.unregister();
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

pub trait RegisterEventHandler<C>
where
    C: AsyncUsbContext,
{
    fn register(self, context: C) -> crate::Result<Box<dyn EventHandlerData<C> + 'static>>;
}

pub trait EventHandlerData<C>: Send + Sync
where
    C: AsyncUsbContext,
{
    // NOTE: Should this be fallible?
    fn unregister(self: Box<Self>);
}

pub trait AsyncUsbContext: UsbContext + 'static {
    fn register_event_handler<T>(&mut self, register_event_handler: T) -> crate::Result<()>
    where
        T: RegisterEventHandler<Self>;
}

impl UsbContext for AsyncContext {
    fn as_raw(&self) -> *mut libusb_context {
        self.context.0.as_raw()
    }
}

impl AsyncUsbContext for AsyncContext {
    fn register_event_handler<T>(&mut self, register_event_handler: T) -> crate::Result<()>
    where
        T: RegisterEventHandler<Self>,
    {
        let ctx_ptr = self.as_raw();

        let mut map = EVENT_DATA_HANDLER_MAP
            .get_or_init(EventHandlerDataMap::default)
            .inner
            .lock()
            .unwrap();

        if let Some(handler_data) = map.remove(&ctx_ptr) {
            handler_data.unregister();
        }

        let handler_data = register_event_handler.register(self.clone())?;
        map.insert(ctx_ptr, handler_data);

        Ok(())
    }
}

impl AsyncUsbContext for GlobalContext {
    fn register_event_handler<T>(&mut self, register_event_handler: T) -> crate::Result<()>
    where
        T: RegisterEventHandler<Self>,
    {
        static EVENT_HANDLER: OnceLock<Option<Box<dyn EventHandlerData<GlobalContext>>>> =
            OnceLock::new();

        if let Err(Some(handler_data)) = EVENT_HANDLER.set(None) {
            handler_data.unregister();
        }

        let handler_data = register_event_handler.register(GlobalContext::default())?;
        EVENT_HANDLER.set(Some(handler_data)).ok();
        Ok(())
    }
}

impl AsyncContext {
    pub fn new<T>(register_event_handler: T) -> crate::Result<Self>
    where
        T: RegisterEventHandler<Self>,
    {
        let ctx = Context::new().map_err(|_| Error::Other("Context creation failed"))?;
        let context = Arc::new(AsyncContextInner(ctx));
        let this = Self { context };
        let handler_data = register_event_handler.register(this.clone())?;

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
        T: RegisterEventHandler<Self>,
    {
        let ctx =
            Context::with_options(opts).map_err(|_| Error::Other("Context creation failed"))?;
        let context = Arc::new(AsyncContextInner(ctx));
        let this = Self { context };
        let handler_data = register_event_handler.register(this.clone())?;

        EVENT_DATA_HANDLER_MAP
            .get_or_init(EventHandlerDataMap::default)
            .inner
            .lock()
            .unwrap()
            .insert(this.as_raw(), handler_data);

        Ok(this)
    }
}
