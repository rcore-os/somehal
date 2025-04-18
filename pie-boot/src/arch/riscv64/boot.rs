use core::arch::naked_asm;

use riscv::register::stvec::{self, Stvec};

use crate::{clean_bss, dbgln};

use super::mmu::init_mmu;

#[naked]
/// The entry point of the kernel.
pub extern "C" fn primary_entry(_hart_id: usize, _fdt_addr: *mut u8) -> ! {
    unsafe {
        naked_asm!(
            "mv      s0, a0",                  // save hartid
            "mv      s1, a1",                  // save DTB pointer

            // Set the stack pointer.
            "la      sp, __boot_stack_bottom",
            "li      t0, {stack_size}",
            "add     sp, sp, t0",

            "mv      a0, s0",
            "mv      a1, s1",
            "call    {get_kcode_va}",
            "mv      s2, a0",    // kcode offset

            "mv      a0, s1",
            "mv      a1, s2",
            "call    {init_mmu}",

            "call    {entry_vma}",
            "mv      t0, a0",

            "mv      a0, s0",  // hartid
            "mv      a1, s2",  // kcode offset
            "mv      a2, s1",  // fdt addr
            "jalr    t0",
            "j       .",
            stack_size = const crate::config::STACK_SIZE,
            get_kcode_va = sym get_kcode_va,
            init_mmu = sym init_mmu,
            entry_vma = sym entry_vma,
        )
    }
}

fn get_kcode_va(hartid: usize, fdt: *mut u8) -> usize {
    unsafe {
        clean_bss();

        let lma = entry_lma();
        let vma = entry_vma();
        let kcode_offset = vma - lma;

        dbgln!("Booting up");
        dbgln!("Entry  LMA     : {}", lma);
        dbgln!("Entry  VMA     : {}", vma);
        dbgln!("Code offset    : {}", kcode_offset);
        dbgln!("Hart           : {}", hartid);
        dbgln!("fdt            : {}", fdt);

        unsafe extern "C" {
            fn trap_vector_base();
        }
        let mut vec = Stvec::from_bits(0);
        vec.set_address(trap_vector_base as usize);
        vec.set_trap_mode(stvec::TrapMode::Direct);
        stvec::write(vec);

        kcode_offset
    }
}

// fn rust_boot(hartid: usize, fdt: *mut u8) -> ! {
//     unsafe {
//         clean_bss();

//         let lma = entry_lma();
//         let vma = entry_vma();
//         let kcode_offset = vma - lma;

//         dbgln!("Booting up");
//         dbgln!("Entry  LMA     : {}", lma);
//         dbgln!("Entry  VMA     : {}", vma);
//         dbgln!("Code offset    : {}", kcode_offset);
//         dbgln!("fdt            : {}", fdt);

//         init_mmu(hartid, fdt, kcode_offset)
//     }
// }

#[naked]
extern "C" fn entry_lma() -> usize {
    unsafe {
        naked_asm!(
            "
    la       a0,  __vma_relocate_entry
    ret"
        )
    }
}
#[naked]
pub extern "C" fn entry_vma() -> usize {
    unsafe {
        naked_asm!(
            "
    .option pic
    la      a0,  __vma_relocate_entry
    ret"
        )
    }
}
