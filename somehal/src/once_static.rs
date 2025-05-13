use core::cell::UnsafeCell;

pub struct OnceStatic<T>(UnsafeCell<Option<T>>);

unsafe impl<T> Sync for OnceStatic<T> {}
unsafe impl<T> Send for OnceStatic<T> {}

use core::ops::Deref;

impl<T> OnceStatic<T> {
    pub const fn new() -> OnceStatic<T> {
        OnceStatic(UnsafeCell::new(None))
    }

    pub unsafe fn get(&self) -> *mut Option<T> {
        self.0.get()
    }

    pub unsafe fn set(&self, val: T) {
        unsafe {
            (*self.0.get()).replace(val);
        }
    }

    pub fn is_init(&self) -> bool {
        unsafe { (*self.0.get()).is_some() }
    }
}

impl<T> Deref for OnceStatic<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            match (*self.0.get()).as_ref() {
                Some(v) => v,
                None => {
                    panic!("{} is not initialized", core::any::type_name::<T>());
                }
            }
        }
    }
}

impl<T> AsRef<T> for OnceStatic<T> {
    fn as_ref(&self) -> &T {
        unsafe { (*self.0.get()).as_ref().unwrap() }
    }
}
