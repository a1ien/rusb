use std::{
    sync::{Arc, Mutex},
    thread,
};

type Signal = Arc<Mutex<bool>>;

pub struct AsyncAwakener {
    signal: Signal,
}

impl Drop for AsyncAwakener {
    fn drop(&mut self) {
        *self.signal.lock().unwrap() = false;
    }
}

impl AsyncAwakener {
    pub fn spawn<F: 'static + Send + FnMut()>(mut func: F) -> Self {
        let signal = Arc::new(Mutex::new(true));

        let thread_signal = signal.clone();
        thread::spawn(move || {
            while *thread_signal.lock().unwrap() {
                func();
            }
        });

        Self { signal }
    }
}
