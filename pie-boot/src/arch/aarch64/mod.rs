use core::arch::{asm, naked_asm};

use crate::{dbgln, mem::new_boot_table};
use aarch64_cpu::{
    asm::{
        barrier::{self, SY, isb},
        wfe,
    },
    registers::*,
};
use page_table_generic::TableGeneric;

use crate::{archif::ArchIf, clean_bss};

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

#[naked]
/// The entry point of the kernel.
pub extern "C" fn primary_entry(_fdt_addr: *mut u8) -> ! {
    unsafe {
        naked_asm!(
            // Save dtb address.
            "MOV      x19, x0",
            // Set the stack pointer.
            "ADRP     x1,  __boot_stack_bottom",
            "ADD      x1, x1, :lo12:__boot_stack_bottom",
            "ADD      x1, x1, {stack_size}",
            "MOV      sp, x1",

            "BL       {switch_to_elx}",
            "MOV      x0,  x19",
            "BL       {entry}",
            "B        .",
            stack_size = const crate::config::STACK_SIZE,
            switch_to_elx = sym switch_to_elx,
            entry = sym rust_boot,
        )
    }
}

fn rust_boot(fdt_addr: *mut u8) -> ! {
    unsafe {
        clean_bss();
        enable_fp();

        #[cfg(early_debug)]
        crate::debug::fdt::init_debugcon(fdt_addr);

        let lma = entry_lma();
        let vma = entry_vma();
        let kcode_offset = vma - lma;

        dbgln!("Booting up");
        dbgln!("Entry  LMA     : {}", lma);
        dbgln!("Entry  VMA     : {}", vma);
        dbgln!("Code offset    : {}", kcode_offset);
        dbgln!("Current EL     : {}", CurrentEL.read(CurrentEL::EL));
        dbgln!("Fdt            : {}", fdt_addr);

        enable_mmu(kcode_offset, fdt_addr)
    }
}

fn enable_mmu(va: usize, fdt: *mut u8) -> ! {
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

#[naked]
extern "C" fn entry_lma() -> usize {
    unsafe {
        naked_asm!(
            "ADRP     x0,  __vma_relocate_entry",
            "ADD      x0, x0, :lo12:__vma_relocate_entry",
            "ret"
        )
    }
}
#[naked]
extern "C" fn entry_vma() -> usize {
    unsafe { naked_asm!("LDR      x0,  =__vma_relocate_entry", "ret") }
}

fn enable_fp() {
    CPACR_EL1.write(CPACR_EL1::FPEN::TrapNothing);
    barrier::isb(barrier::SY);
}

pub struct Arch;

impl ArchIf for Arch {
    fn early_debug_put(byte: u8) {
        #[cfg(early_debug)]
        crate::debug::write_byte(byte);
    }

    fn wait_for_event() {
        wfe();
    }

    type PageTable = Table;

    fn new_pte_with_config(
        config: kmem::region::MemConfig,
    ) -> <Self::PageTable as TableGeneric>::PTE {
        new_pte_with_config(config)
    }
}
