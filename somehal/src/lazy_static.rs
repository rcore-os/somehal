use core::{cell::UnsafeCell, ops::Deref, sync::atomic::AtomicBool};

pub struct LazyStatic<T> {
    value: UnsafeCell<Option<T>>,
    initialized: AtomicBool,
}

unsafe impl<T> Send for LazyStatic<T> {}
unsafe impl<T> Sync for LazyStatic<T> {}

impl<T> LazyStatic<T> {
    pub const fn new() -> Self {
        Self {
            value: UnsafeCell::new(None),
            initialized: AtomicBool::new(false),
        }
    }

    pub const fn with_default(value: T) -> Self {
        Self {
            value: UnsafeCell::new(Some(value)),
            initialized: AtomicBool::new(false),
        }
    }

    pub unsafe fn clean(&self) {
        unsafe {
            *self.value.get() = None;
            self.initialized
                .store(false, core::sync::atomic::Ordering::Release);
        }
    }

    pub fn init(&self, value: T) {
        self.try_init(value)
            .expect("StaticOnce already initialized");
    }

    pub fn try_init(&self, value: T) -> Result<(), &'static str> {
        if self
            .initialized
            .swap(true, core::sync::atomic::Ordering::Release)
        {
            return Err("StaticOnce has already been initialized");
        }
        unsafe {
            *self.value.get() = Some(value);
        }
        Ok(())
    }

    /// # Safety
    /// Must be called only once after initialization.
    /// Make sure the operation is thread-safe.
    pub unsafe fn edit<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        unsafe {
            let value = &mut *self.value.get();
            f(value.as_mut().expect("StaticOnce not initialized"))
        }
    }
}

impl<T> Deref for LazyStatic<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        let r = unsafe { &*self.value.get() };
        r.as_ref().expect("StaticOnce not initialized")
    }
}
