use core::cell::OnceCell;

use sync::spin::{Guard, SpinLock};

use crate::raw::write::RawWriter;

pub struct Writer {
    inner: SpinLock<OnceCell<RawWriter>>,
}

impl Writer {
    pub const fn new() -> Writer {
        Writer {
            inner: SpinLock::new(OnceCell::new()),
        }
    }

    pub fn initialize(&self, value: RawWriter) {
        self.inner.lock().get_or_init(|| value);
    }

    pub fn locked(&self) -> Guard<OnceCell<RawWriter>> {
        self.inner.lock()
    }
}

impl Default for Writer {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl Send for Writer {}

unsafe impl Sync for Writer {}
