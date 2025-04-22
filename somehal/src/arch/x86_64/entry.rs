use crate::{mem::boot::set_kcode_va_offset, println};

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __vma_relocate_entry() -> ! {
    unsafe {
        // set_kcode_va_offset(KCODE_OFFSET);
    }
    println!("MMU ready");

    loop {}
}
