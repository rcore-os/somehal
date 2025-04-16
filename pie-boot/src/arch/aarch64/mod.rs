use core::arch::{asm, naked_asm};

use crate::dbgln;
use aarch64_cpu::{
    asm::{barrier, wfe},
    registers::*,
};
use mmu::enable_mmu;
use page_table_generic::TableGeneric;

use crate::{archif::ArchIf, clean_bss};

mod mmu;

pub use mmu::Table;

#[naked]
/// The entry point of the kernel.
pub extern "C" fn primary_entry(_fdt_addr: *mut u8) -> ! {
    unsafe {
        naked_asm!(
            "MOV      x19, x0",
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
        dbgln!("Entry  LMA : {}", lma);
        dbgln!("Entry  VMA : {}", vma);
        dbgln!("Code offset: {}", kcode_offset);
        dbgln!("Current EL : {}", CurrentEL.read(CurrentEL::EL));
        dbgln!("fdt        : {}", fdt_addr);

        enable_mmu(kcode_offset, fdt_addr)
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

/// Switch to EL1.
#[cfg(not(feature = "vm"))]
fn switch_to_elx() {
    use core::arch::asm;

    SPSel.write(SPSel::SP::ELx);
    SP_EL0.set(0);
    let current_el = CurrentEL.read(CurrentEL::EL);
    if current_el >= 2 {
        if current_el == 3 {
            // Set EL2 to 64bit and enable the HVC instruction.
            SCR_EL3.write(
                SCR_EL3::NS::NonSecure + SCR_EL3::HCE::HvcEnabled + SCR_EL3::RW::NextELIsAarch64,
            );
            // Set the return address and exception level.
            SPSR_EL3.write(
                SPSR_EL3::M::EL1h
                    + SPSR_EL3::D::Masked
                    + SPSR_EL3::A::Masked
                    + SPSR_EL3::I::Masked
                    + SPSR_EL3::F::Masked,
            );
            unsafe {
                asm!(
                "adr    x2, {}",
                "mov    x0, x19",
                "msr    elr_el3, x2",
                 sym primary_entry
                    );
            }
        }
        // Disable EL1 timer traps and the timer offset.
        CNTHCTL_EL2.modify(CNTHCTL_EL2::EL1PCEN::SET + CNTHCTL_EL2::EL1PCTEN::SET);
        CNTVOFF_EL2.set(0);
        // Set EL1 to 64bit.
        HCR_EL2.write(HCR_EL2::RW::EL1IsAarch64);
        // Set the return address and exception level.
        SPSR_EL2.write(
            SPSR_EL2::M::EL1h
                + SPSR_EL2::D::Masked
                + SPSR_EL2::A::Masked
                + SPSR_EL2::I::Masked
                + SPSR_EL2::F::Masked,
        );
        unsafe {
            asm!(
                "
            mov     x8, sp
            msr     sp_el1, x8
            MOV     x0, x19
            adr     x2, {}
            msr     elr_el2, x2
            eret
            " , 
            sym primary_entry
            )
        };
    }
}

/// Switch to EL2.
#[cfg(feature = "vm")]
fn switch_to_elx() {
    SPSel.write(SPSel::SP::ELx);
    let current_el = CurrentEL.read(CurrentEL::EL);
    if current_el == 3 {
        SCR_EL3.write(
            SCR_EL3::NS::NonSecure + SCR_EL3::HCE::HvcEnabled + SCR_EL3::RW::NextELIsAarch64,
        );
        SPSR_EL3.write(
            SPSR_EL3::M::EL2h
                + SPSR_EL3::D::Masked
                + SPSR_EL3::A::Masked
                + SPSR_EL3::I::Masked
                + SPSR_EL3::F::Masked,
        );
        ELR_EL3.set(LR.get());
        aarch64_cpu::asm::eret();
    }

    // Set EL1 to 64bit.
    // Enable `IMO` and `FMO` to make sure that:
    // * Physical IRQ interrupts are taken to EL2;
    // * Virtual IRQ interrupts are enabled;
    // * Physical FIQ interrupts are taken to EL2;
    // * Virtual FIQ interrupts are enabled.
    HCR_EL2.modify(
        HCR_EL2::VM::Enable
            + HCR_EL2::RW::EL1IsAarch64
            + HCR_EL2::IMO::EnableVirtualIRQ // Physical IRQ Routing.
            + HCR_EL2::FMO::EnableVirtualFIQ // Physical FIQ Routing.
            + HCR_EL2::TSC::EnableTrapEl1SmcToEl2,
    );
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
        mmu::new_pte_with_config(config)
    }
}
