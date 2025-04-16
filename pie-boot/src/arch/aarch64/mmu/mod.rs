use core::arch::asm;

use aarch64_cpu::asm::barrier::{SY, isb};

use crate::{dbgln, mem::new_boot_table};

cfg_match! {
    feature = "vm" => {
        mod el2;
        pub use el2::*;
    }
    _ => {
        mod el1;
        pub use el1::*;
    }
}

pub fn enable_mmu(va: usize, fdt: *mut u8) -> ! {
    setup_table_regs();
    #[cfg(fdt)]
    let fdt_size = crate::debug::fdt::fdt_size(fdt);
    #[cfg(not(fdt))]
    let fdt_size = 0;

    let table = new_boot_table(fdt_size, va);

    dbgln!("Set kernel table {}", table.raw());
    set_table(table);
    flush_tlb(None);

    let jump_to: *mut u8;
    unsafe {
        asm!("LDR {0}, =__vma_relocate_entry",
            out(reg) jump_to,
        );
        dbgln!("relocate to pc: {}", jump_to);
        // Enable the MMU and turn on I-cache and D-cache
        setup_sctlr();
        isb(SY);
        asm!(
            "MOV      x0,  {kcode_va}",
            "MOV      x1,  {fdt}",
            "MOV      x8,  {jump}",
            "BLR      x8",
            "B       .",
            kcode_va = in(reg) va,
            fdt = in(reg) fdt,
            jump = in(reg) jump_to,
            options(nostack, noreturn)
        )
    }
}
