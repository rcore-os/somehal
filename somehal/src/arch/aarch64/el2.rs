use core::arch::asm;

use aarch64_cpu::{asm::barrier, registers::*};
use pie_boot_macros::start_code;

#[start_code]
pub fn switch_to_elx(bootargs: usize) {
    SPSel.write(SPSel::SP::ELx);
    SP_EL0.set(0);
    let current_el = CurrentEL.read(CurrentEL::EL);
    let ret = sym_lma!(super::_start_secondary);
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
}
