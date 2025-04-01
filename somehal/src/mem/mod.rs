pub use kmem::*;

#[link_boot::link_boot]
mod _m {
    use core::ptr::slice_from_raw_parts_mut;

    fn bss() -> &'static mut [u8] {
        unsafe extern "C" {
            fn __start_bss();
            fn __stop_bss();
        }
        unsafe {
            &mut *slice_from_raw_parts_mut(
                __start_bss as _,
                __stop_bss as usize - __start_bss as usize,
            )
        }
    }

    pub(crate) unsafe fn clean_bss() {
        bss().fill(0);
    }

    pub(crate) fn entry_addr() -> usize {
        unsafe extern "C" {
            fn __start_BootText();
        }

        __start_BootText as usize
    }
}
