use crate::{arch::uart16550, mem::boot::set_kcode_va_offset, println};

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __vma_relocate_entry(kcode_offset: usize, mbi: usize) -> ! {
    unsafe {
        set_kcode_va_offset(kcode_offset);
    }
    uart16550::init();

    println!("MMU ready");

    loop {}
}
