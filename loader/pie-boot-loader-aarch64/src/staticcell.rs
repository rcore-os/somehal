use core::cell::UnsafeCell;

#[repr(transparent)]
pub struct StaticCell<T>(UnsafeCell<T>);

impl<T> StaticCell<T> {
    pub const fn new(value: T) -> Self {
        StaticCell(UnsafeCell::new(value))
    }

    #[allow(clippy::mut_from_ref)]
    pub fn as_mut(&self) -> &mut T {
        unsafe { &mut *self.0.get() }
    }
}

impl<T> core::ops::Deref for StaticCell<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0.get() }
    }
}

unsafe impl<T> Send for StaticCell<T> {}
unsafe impl<T> Sync for StaticCell<T> {}
