use core::ptr::slice_from_raw_parts_mut;

pub use kmem::*;

unsafe extern "C" {
    static _sbss: *mut u8;
    static _ebss: *mut u8;
}

pub(crate) unsafe fn clean_bss() {
    unsafe { &mut *slice_from_raw_parts_mut(_sbss, _ebss as usize - _sbss as usize) }.fill(0);
}
