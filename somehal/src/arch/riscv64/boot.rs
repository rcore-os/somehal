use core::arch::naked_asm;

#[cfg(target_pointer_width = "64")]
const XLEN: usize = 0x200000;
#[cfg(target_pointer_width = "32")]
const XLEN: usize = 0x400000;

const FLAG_LE: usize = 0b0;

const HEADER_VERSION_MAJOR: usize = 0;
const HEADER_VERSION_MINOR: usize = 2;
const HEADER_VERSION: usize = (HEADER_VERSION_MAJOR << 16) | HEADER_VERSION_MINOR;

#[naked]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.boot.header")]
/// The entry point of the kernel.
pub unsafe extern "C" fn _start() -> ! {
    unsafe {
        naked_asm!(
            // code0/code1
            "j  {entry}",
            ".word 0",
            // Image load offset, little endian
            ".dword {offset}",
            // Image size, little endian
            ".dword  __kernel_load_size",
            // flags
            ".dword  {flags}",
            ".word   {version}",
            ".word   0",
            // Reserved fields
            ".dword 0",
            // Magic number, little endian, "RISCV"
            ".dword 0x5643534952",
            // Magic number 2, little endian, "RSC\x05"
            ".word  0x05435352",
            ".word  0",
            offset = const XLEN,
            flags = const FLAG_LE ,
            version = const HEADER_VERSION,
            entry = sym pie_boot::primary_entry,
            // entry = sym entry,
        )
    }
}
