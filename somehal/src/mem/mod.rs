pub use kmem::*;

use crate::consts::STACK_SIZE;

#[link_boot::link_boot]
mod _m {
    use core::ptr::slice_from_raw_parts_mut;

    use crate::{ArchIf, arch::Arch};

    static mut KCODE_VA_OFFSET: usize = 0;

    pub(crate) unsafe fn set_kcode_va_offset(offset: usize) {
        unsafe { KCODE_VA_OFFSET = offset };
    }

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

    pub(crate) fn stack_top() -> usize {
        unsafe extern "C" {
            fn __stack_bottom();
        }

        __stack_bottom as usize + STACK_SIZE
            - if Arch::is_mmu_enabled() {
                kcode_offset()
            } else {
                0
            }
    }

    pub(crate) fn kcode_offset() -> usize {
        unsafe { KCODE_VA_OFFSET }
    }
}
