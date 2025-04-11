use core::arch::{asm, naked_asm};

use aarch64_cpu::{asm::barrier, registers::*};

use crate::clean_bss;

mod mmu;
#[cfg(feature="early-debug")]
mod debug;

#[naked]
/// The entry point of the kernel.
pub extern "C" fn primary_entry(_fdt_addr: *mut u8) -> usize {
    unsafe {
        naked_asm!(
            "MOV      x19, x0",
            "BL       {switch_to_elx}",
            "LDR      x0,  =__vma_relocate_entry",
            "ADRP     x1,  __vma_relocate_entry",
            "ADD      x1, x1, :lo12:__vma_relocate_entry",
            "SUB      x0, x0, x1",
            "MOV      x1,  x19",
            "BL       {entry}",
            switch_to_elx = sym switch_to_elx,
            entry = sym rust_boot,
        )
    }
}

fn rust_boot(kcode_offset: usize, fdt_addr: *mut u8) -> ! {
    unsafe {
        clean_bss();
        enable_fp();

        asm!("", options(noreturn))
    }
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
