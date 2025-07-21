use core::arch::asm;

use aarch64_cpu::{asm::barrier, registers::*};
use pie_boot_macros::start_code;

#[start_code]
pub fn switch_to_elx(bootargs: usize) {
    SPSel.write(SPSel::SP::ELx);
    SP_EL0.set(0);
    let current_el = CurrentEL.read(CurrentEL::EL);
    let ret = sym_lma!(super::_start_secondary);
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

            ELR_EL3.set(ret as _);
            barrier::isb(barrier::SY);
            unsafe {
                asm!(
                    "
                    mov x0, {}
                    eret
                    ",
                    in(reg) bootargs,
                    options(nostack, noreturn),
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
            ELR_EL2.set(ret as _);
            barrier::isb(barrier::SY);

            asm!(
                "
            mov x0, {}
            eret
            ",
                in(reg) bootargs,
                options(nostack, noreturn),
            )
        };
    }
}
