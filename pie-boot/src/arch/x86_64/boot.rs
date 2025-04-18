use core::arch::{global_asm, naked_asm};

use crate::{clean_bss, dbgln};

use super::mmu::enable_mmu;

const BOOT_STACK_SIZE: usize = 0x4000;

const MAGIC: i32 = 0x1BADB002;

const MODULEALIGN: i32 = 1 << 0;
const MEMINFO: i32 = 1 << 1;
const FLAGS: i32 = MODULEALIGN | MEMINFO;
const CHECKSUM: i32 = -(MAGIC + FLAGS);

#[naked]
#[unsafe(no_mangle)]
#[repr(align(4))]
#[unsafe(link_section = ".text.boot.header")]
pub extern "C" fn __header() -> ! {
    unsafe {
        naked_asm!(
            "
.code32
.int  {magic}
.int  {flags}
.int  {checksum}
        ",
        magic = const MAGIC,
        flags = const FLAGS,
        checksum = const CHECKSUM,
        )
    }
}

global_asm!(
    include_str!("boot.asm"),
    // entry = sym primary_entry,
);

/// The entry point of the kernel.
pub extern "C" fn primary_entry() -> ! {
    unsafe extern "C" {
        fn __vma_relocate_entry() -> !;
    }

    unsafe {
        clean_bss();
        __vma_relocate_entry()
    }
}
#[naked]
/// The entry point of the kernel.
pub extern "C" fn secondary_entry(_hart_id: usize, _fdt_addr: *mut u8) -> ! {
    unsafe {
        naked_asm!(
            "",
            // stack_size = const crate::config::STACK_SIZE,
            // entry = sym rust_boot,
        )
    }
}

fn rust_boot(hartid: usize, fdt: *mut u8) -> ! {
    unsafe {
        clean_bss();

        let lma = entry_lma();
        let vma = entry_vma();
        let kcode_offset = vma - lma;

        dbgln!("Booting up");
        dbgln!("Entry  LMA     : {}", lma);
        dbgln!("Entry  VMA     : {}", vma);
        dbgln!("Code offset    : {}", kcode_offset);
        dbgln!("fdt            : {}", fdt);

        enable_mmu(hartid, fdt, kcode_offset)
    }
}

#[naked]
extern "C" fn entry_lma() -> usize {
    unsafe { naked_asm!("") }
}
#[naked]
pub extern "C" fn entry_vma() -> usize {
    unsafe { naked_asm!("") }
}
