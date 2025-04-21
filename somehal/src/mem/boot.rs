static mut KCODE_VA_OFFSET: usize = 0;

pub unsafe fn set_kcode_va_offset(offset: usize) {
    unsafe { KCODE_VA_OFFSET = offset };
}

pub fn kcode_offset() -> usize {
    unsafe { KCODE_VA_OFFSET }
}
