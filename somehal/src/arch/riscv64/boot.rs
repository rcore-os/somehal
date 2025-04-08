use riscv::register::stvec;

use crate::{
    consts::{KERNEL_ENTRY_VADDR, KERNEL_STACK_SIZE},
    fdt::set_fdt_ptr,
    mem::boot::*,
};

/// The earliest entry point for the primary CPU.
#[naked]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.boot.header")]
unsafe extern "C" fn _start() -> ! {
    // a0 = hartid
    // a1 = dtb
    unsafe {
        core::arch::naked_asm!("
        auipc   t2, 0
        li      t0, {kernel_load_addr}
        sub     t2, t0, t2
        mv      s0, a0                  // save hartid
        mv      s1, a1                  // save DTB pointer
        la      sp, __stack_bottom
        li      t0, {boot_stack_size}
        add     sp, sp, t0              // setup boot stack

        mv      a0, s0
        mv      a1, s1
        mv      a2, t2
        call    {entry}
        ",
            kernel_load_addr = const KERNEL_ENTRY_VADDR,
            boot_stack_size = const KERNEL_STACK_SIZE,
            entry = sym boot_entry,
        )
    }
}

#[link_boot::link_boot]
mod _m {
    use crate::{arch::paging::enable_mmu, dbgln};

    fn boot_entry(_hartid: usize, fdt: *mut u8, kcode_va: usize) -> ! {
        unsafe {
            clean_bss();
            set_kcode_va_offset(kcode_va);
            set_fdt_ptr(fdt);

            dbgln!("Booting up");
            dbgln!("Entry      : {}", KERNEL_ENTRY_VADDR - kcode_va);
            dbgln!("Code offset: {}", kcode_offset());
            dbgln!("fdt        : {}", fdt);
            dbgln!("fdt size   : {}", crate::fdt::fdt_size());

            unsafe extern "C" {
                fn trap_vector_base();
            }
            set_trap_vector_base(trap_vector_base as usize);

            enable_mmu()
        }
    }
}
/// Writes Supervisor Trap Vector Base Address Register (`stvec`).
#[inline]
pub fn set_trap_vector_base(stvec: usize) {
    let mut v = stvec::Stvec::from_bits(0);
    v.set_address(stvec);
    v.set_trap_mode(stvec::TrapMode::Direct);
    unsafe {
        stvec::write(v);
    }
}
