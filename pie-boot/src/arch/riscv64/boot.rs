use core::arch::{global_asm, naked_asm};

use riscv::register::stvec::{self, Stvec};

use crate::{
    arch::debug_init,
    dbgln,
    mem::{clean_bss, edit_boot_info, init_phys_allocator, set_fdt_ptr},
};

use super::mmu::init_mmu;

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
        entry = sym primary_entry,
    )
}

#[naked]
unsafe extern "C" fn primary_entry(_hart_id: usize, _fdt_addr: *mut u8) -> ! {
    naked_asm!(
        "mv      s0, a0",                  // save hartid
        "mv      s1, a1",                  // save DTB pointer

        // Set the stack pointer.
        "la      sp, __kernel_code_end",
        "li      t0, {stack_size}",
        "add     sp, sp, t0",

        "mv      a0, s0",
        "mv      a1, s1",
        "call    {setup}",
        "mv      s2, a0",    // return kcode offset

        "call    {init_mmu}",

        "mv      a0, s0",  // hartid
        "mv      a1, s2",  // kcode offset
        "mv      a2, s1",  // fdt addr
        "call    {setup_boot_info}",

        "call    {entry_vma}",
        "mv      t0, a0",

        "mv      gp, zero",

        "mv      a0, s0",
        "jalr    t0",
        "j       .",
        stack_size = const crate::config::BOOT_STACK_SIZE,
        setup = sym setup,
        init_mmu = sym init_mmu,
        setup_boot_info = sym setup_boot_info,
        entry_vma = sym relocate_vma,
    )
}
fn setup(hartid: usize, fdt: *mut u8) -> usize {
    unsafe {
        clean_bss();
        debug_init();
        let lma = entry_lma();

        let vma = entry_vma();
        let kcode_offset = vma - lma;

        dbgln!("Booting up");
        dbgln!("Entry  LMA     : {}", lma);
        dbgln!("Entry  VMA     : {}", vma);
        dbgln!("Code offset    : {}", kcode_offset);
        dbgln!("Hart           : {}", hartid);
        dbgln!("fdt            : {}", fdt);

        set_fdt_ptr(fdt);

        unsafe extern "C" {
            fn trap_vector_base();
        }
        let mut vec = Stvec::from_bits(0);
        vec.set_address(trap_vector_base as usize);
        vec.set_trap_mode(stvec::TrapMode::Direct);
        stvec::write(vec);

        init_phys_allocator(0);

        kcode_offset
    }
}

fn setup_boot_info(hartid: usize, kcode_offset: usize) {
    unsafe {
        edit_boot_info(|info| {
            info.cpu_id = hartid;
            info.kcode_offset = kcode_offset;
        });
    }
}

#[naked]
unsafe extern "C" fn entry_lma() -> usize {
    naked_asm!(
        "
    la       a0,  __vma_relocate_entry
    ret"
    )
}

/// The entry point of the kernel.
///
/// # Safety
#[naked]
pub unsafe extern "C" fn entry_vma() -> usize {
    naked_asm!(
        "
    .option pic
    la      a0,  __vma_relocate_entry
    ret"
    )
}

/// The entry point of the kernel.
///
/// # Safety
#[naked]
pub unsafe extern "C" fn relocate_vma() -> usize {
    naked_asm!(
        "
    .option pic
    la      a0,  {}
    ret",
    sym crate::relocate
    )
}
