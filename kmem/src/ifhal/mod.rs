/// Set the offset of the kernel code virtual address.
///
/// # Safety
///
/// This function is unsafe because it can cause undefined behavior if the offset is not valid.
pub unsafe fn set_kcode_va_offset(offset: usize) {
    unsafe { super::space::KCODE_VA_OFFSET = offset };
}
