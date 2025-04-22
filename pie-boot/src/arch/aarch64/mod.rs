use core::{
    arch::{asm, naked_asm},
    ptr::NonNull,
};

use crate::paging::TableGeneric;
use crate::{
    dbgln,
    mem::{edit_boot_info, init_phys_allocator, new_boot_table},
};
use aarch64_cpu::{
    asm::{
        barrier::{self, SY, isb},
        wfe,
    },
    registers::*,
};

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

const FLAG_LE: usize = 0b0;
const FLAG_PAGE_SIZE_4K: usize = 0b10;
const FLAG_ANY_MEM: usize = 0b1000;

#[unsafe(naked)]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.boot.header")]
/// The header of the kernel.
/// 
/// # Safety
pub unsafe extern "C" fn _start() -> ! {
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
        entry = sym primary_entry,
    )
}

#[unsafe(naked)]
unsafe extern "C" fn primary_entry(_fdt_addr: *mut u8) -> ! {
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

        init_phys_allocator();

        edit_boot_info(|info| {
            info.cpu_id = (MPIDR_EL1.get() as usize) & 0xffffff;
            info.kcode_offset = kcode_offset;
            info.fdt = NonNull::new(fdt_addr);
        });

        enable_mmu(kcode_offset)
    }
}

fn enable_mmu(va: usize) -> ! {
    setup_table_regs();

    let table = new_boot_table(va);

    dbgln!("Set kernel table {}", table.raw());
    set_table(table);
    flush_tlb(None);

    let jump_to: *mut u8;
    unsafe {
        asm!("LDR {0}, ={entry}",
            out(reg) jump_to,
            entry = sym crate::relocate,
        );
        dbgln!("relocate to pc: {}", jump_to);
        // Enable the MMU and turn on I-cache and D-cache
        setup_sctlr();
        isb(SY);
        asm!(
            "MOV      x8,  {jump}",
            "BLR      x8",
            "B       .",
            jump = in(reg) jump_to,
            options(nostack, noreturn)
        )
    }
}

#[unsafe(naked)]
unsafe extern "C" fn entry_lma() -> usize {
    naked_asm!(
        "ADRP     x0,  __vma_relocate_entry",
        "ADD      x0, x0, :lo12:__vma_relocate_entry",
        "ret"
    )
}

#[unsafe(naked)]
unsafe extern "C" fn entry_vma() -> usize {
    naked_asm!("LDR      x0,  =__vma_relocate_entry", "ret")
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
        config: kmem_region::region::MemConfig,
    ) -> <Self::PageTable as TableGeneric>::PTE {
        new_pte_with_config(config)
    }
}
