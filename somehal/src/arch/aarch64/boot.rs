use core::arch::naked_asm;

const FLAG_LE: usize = 0b0;
const FLAG_PAGE_SIZE_4K: usize = 0b10;
const FLAG_ANY_MEM: usize = 0b1000;

#[naked]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.boot.header")]
/// The entry point of the kernel.
pub unsafe extern "C" fn _start() -> ! {
    unsafe {
        naked_asm!(
            // code0/code1
            "nop",
            "bl {entry}",
            // text_offset
            ".quad 0",
            // image_size
            ".quad __kernel_load_size",
            // flags
            ".quad {flags}",
            // Reserved fields
            ".quad 0",
            ".quad 0",
            ".quad 0",
            // magic - yes 0x644d5241 is the same as ASCII string "ARM\x64"
            ".ascii \"ARM\\x64\"",
            // Another reserved field at the end of the header
            ".byte 0, 0, 0, 0",
            flags = const FLAG_LE | FLAG_PAGE_SIZE_4K | FLAG_ANY_MEM,
            entry = sym pie_boot::primary_entry,
        )
    }
}
