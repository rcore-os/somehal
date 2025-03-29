use core::ptr::slice_from_raw_parts_mut;

pub(crate) static mut KCODE_VA_OFFSET: usize = 0;

unsafe extern "C" {
    static _sbss: *mut u8;
    static _ebss: *mut u8;
}

/// # Safety
///
/// call at boot time
pub unsafe fn clean_bss() {
    unsafe { &mut *slice_from_raw_parts_mut(_sbss, _ebss as usize - _sbss as usize) }.fill(0);
}
