use core::cell::UnsafeCell;

pub struct OnceStatic<T>(UnsafeCell<Option<T>>);

unsafe impl<T> Sync for OnceStatic<T> {}
unsafe impl<T> Send for OnceStatic<T> {}

#[link_boot::link_boot]
mod _m {
    use core::ops::Deref;

    impl<T> OnceStatic<T> {
        pub const fn new() -> OnceStatic<T> {
            OnceStatic(UnsafeCell::new(None))
        }

        pub unsafe fn get(&self) -> *mut Option<T> {
            self.0.get()
        }
    }

    impl<T> Deref for OnceStatic<T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            unsafe { (*self.0.get()).as_ref().unwrap() }
        }
    }
}
