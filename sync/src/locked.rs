use core::cell::OnceCell;

use crate::spin::{Guard, SpinLock};

pub struct Locked<T> {
    // todo: replace spinlock with a more sophisticated lock lol
    inner: SpinLock<OnceCell<T>>,
}

impl<T> Locked<T> {
    pub const fn new() -> Locked<T> {
        Locked {
            inner: SpinLock::new(OnceCell::new()),
        }
    }

    pub fn initialize(&self, value: T) {
        self.inner.lock().get_or_init(|| value);
    }

    pub fn locked(&self) -> Guard<OnceCell<T>> {
        self.inner.lock()
    }
}

impl<T> Default for Locked<T> {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl<T> Send for Locked<T> {}

unsafe impl<T> Sync for Locked<T> {}
