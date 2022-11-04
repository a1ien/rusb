use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::Duration;

use rusb::{Error, UsbContext};
use rusb::ffi::*;

struct EventThread {
    thread: Option<JoinHandle<Result<(), Error>>>,
    should_quit: Arc<AtomicBool>,
}

impl EventThread {
    fn new(context: &mut rusb::Context) -> Self {
        let thread_context = context.clone();
        let tx = Arc::new(AtomicBool::new(false));
        let rx = tx.clone();

        let thread = std::thread::spawn(move || -> Result<(), Error> {
            while !rx.load(Ordering::SeqCst) {
                thread_context.handle_events(Some(Duration::from_millis(0)))?;
            }

            Ok(())
        });

        Self {
            thread: Some(thread),
            should_quit: tx,
        }
    }
}

impl Drop for EventThread {
    fn drop(&mut self) {
        self.should_quit.store(true, Ordering::SeqCst);

        let _ = self.thread
            .take()
            .map(|thread| thread.join());
    }
}

/// A `libusb` context with a dedicated thread to handle events in the background.
#[derive(Clone)]
pub struct Context {
    inner: rusb::Context,
    _thread: Arc<EventThread>,
}

impl Context {
    /// Opens a new `libusb` context and spawns a thread to handle events in the background for
    /// that context.
    pub fn new() -> Result<Self, Error> {
        let mut inner = rusb::Context::new()?;
        let thread = EventThread::new(&mut inner);

        Ok(Self {
            inner,
            _thread: Arc::new(thread),
        })
    }
}

impl UsbContext for Context {
    fn as_raw(&self) -> *mut libusb_context {
        self.inner.as_raw()
    }
}
