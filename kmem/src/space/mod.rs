pub(crate) static mut KCODE_VA_OFFSET: usize = 0;

unsafe extern "C" {
    static __start_bss: *mut u8;
}
