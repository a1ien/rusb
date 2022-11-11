use rusb::{Device, Error, UsbContext};

use futures::{
    channel::mpsc::{channel, Receiver, Sender},
    SinkExt, StreamExt,
};
use std::borrow::Borrow;

/// Events retrieved by polling the [`Registration`] whenever new USB devices arrive or existing
/// USB devices leave.
#[derive(Debug)]
pub enum HotplugEvent<T: UsbContext> {
    /// A new device arrived.
    Arrived(Device<T>),
    /// The specified device left.
    Left(Device<T>),
}

/// Builds hotplug [`Registration`] with custom configuration values.
pub struct HotplugBuilder {
    inner: rusb::HotplugBuilder,
}

impl HotplugBuilder {
    /// Returns a new builder with no filter. Devices can optionally be filtered by
    /// [`HotplugBuilder::vendor_id`], [`HotplugBuilder::product_id`] and
    /// [`HotplugBuilder::class`].
    ///
    /// Registration is done by calling [`HotplugBuilder::register`].
    pub fn new() -> Self {
        Self {
            inner: rusb::HotplugBuilder::new(),
        }
    }

    /// Devices can optionally be filtered by their vendor ID.
    pub fn vendor_id(&mut self, vendor_id: u16) -> &mut Self {
        self.inner.vendor_id(vendor_id);
        self
    }

    /// Devices can optionally be filtered by their product ID.
    pub fn product_id(&mut self, product_id: u16) -> &mut Self {
        self.inner.product_id(product_id);
        self
    }

    /// Devices can optionally be filtered by their class.
    pub fn class(&mut self, class: u8) -> &mut Self {
        self.inner.class(class);
        self
    }

    /// If `enumerate` is `true`, then devices that are already connected will cause the
    /// [`Registration`] to return [`HotplugEvent::Arrived`] events for them.
    pub fn enumerate(&mut self, enumerate: bool) -> &mut Self {
        self.inner.enumerate(enumerate);
        self
    }

    /// Registers the hotplug configuration and returns a [`Registration`] object that can be
    /// polled for [`HotplugEvents`](HotplugEvent).
    pub fn register<U: rusb::UsbContext + 'static, T: Borrow<U>>(
        &mut self,
        context: T,
    ) -> Result<Registration<U>, Error> {
        let (tx, rx): (Sender<HotplugEvent<U>>, Receiver<HotplugEvent<U>>) = channel(1);

        let hotplug = Box::new(Hotplug { tx });

        let inner = self.inner.register(context, hotplug)?;

        Ok(Registration { _inner: inner, rx })
    }
}

struct Hotplug<T: UsbContext> {
    tx: Sender<HotplugEvent<T>>,
}

impl<T: UsbContext> rusb::Hotplug<T> for Hotplug<T> {
    fn device_arrived(&mut self, device: Device<T>) {
        futures::executor::block_on(async {
            self.tx.send(HotplugEvent::Arrived(device)).await.unwrap();
        })
    }

    fn device_left(&mut self, device: Device<T>) {
        futures::executor::block_on(async {
            self.tx.send(HotplugEvent::Left(device)).await.unwrap();
        });
    }
}

/// The hotplug registration which can be polled for [`HotplugEvents`](HotplugEvent).
pub struct Registration<T: UsbContext> {
    _inner: rusb::Registration<T>,
    rx: Receiver<HotplugEvent<T>>,
}

impl<T: UsbContext> Registration<T> {
    /// Creates a future to await the next [`HotplugEvent`].
    pub async fn next_event(&mut self) -> Option<HotplugEvent<T>> {
        self.rx.next().await
    }
}
