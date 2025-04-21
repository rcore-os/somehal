use core::{arch::naked_asm, ptr::NonNull};

use riscv::register::stvec::{self, Stvec};

use crate::{
    BootInfo,
    arch::debug_init,
    clean_bss, dbgln,
    mem::{boot_info_addr, edit_boot_info, init_phys_allocator},
};

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
            "call    {setup}",
            "mv      s2, a0",    // return kcode offset

            "call    {init_mmu}",

            "mv      a0, s0",  // hartid
            "mv      a1, s2",  // kcode offset
            "mv      a2, s1",  // fdt addr
            "call    {setup_boot_info}",
            "mv      s0, a0",

            "call    {entry_vma}",
            "mv      t0, a0",

            "mv      gp, zero",

            "mv      a0, s0",
            "jalr    t0",
            "j       .",
            stack_size = const crate::config::STACK_SIZE,
            setup = sym setup,
            init_mmu = sym init_mmu,
            setup_boot_info = sym setup_boot_info,
            entry_vma = sym entry_vma,
        )
    }
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

        unsafe extern "C" {
            fn trap_vector_base();
        }
        let mut vec = Stvec::from_bits(0);
        vec.set_address(trap_vector_base as usize);
        vec.set_trap_mode(stvec::TrapMode::Direct);
        stvec::write(vec);

        init_phys_allocator();

        kcode_offset
    }
}

fn setup_boot_info(hartid: usize, kcode_offset: usize, fdt: *mut u8) -> *const BootInfo {
    unsafe {
        edit_boot_info(|info| {
            info.cpu_id = hartid;
            info.kcode_offset = kcode_offset;
            info.fdt = NonNull::new(fdt);
        });
    }
    boot_info_addr()
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
