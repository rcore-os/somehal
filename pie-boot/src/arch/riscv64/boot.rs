use core::arch::naked_asm;

use riscv::register::stvec::{self, Stvec};

use crate::{clean_bss, dbgln};

use super::mmu::enable_mmu;

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
            "call    {entry}",
            stack_size = const crate::config::STACK_SIZE,
            entry = sym rust_boot,
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

        unsafe extern "C" {
            fn trap_vector_base();
        }
        let mut vec = Stvec::from_bits(0);
        vec.set_address(trap_vector_base as usize);
        vec.set_trap_mode(stvec::TrapMode::Direct);
        stvec::write(vec);

        enable_mmu(hartid, fdt, kcode_offset)
    }
}

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
