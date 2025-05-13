use aarch64_cpu::{asm::barrier, registers::*};

use crate::{
    arch::paging::{flush_tlb, set_mair},
    mp::CpuOnArg,
};

pub unsafe fn switch_to_elx() {
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
            ELR_EL3.set(LR.get());
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
            core::arch::asm!(
                "
                mov     x8, sp
                msr     sp_el1, x8"
            )
        };
        ELR_EL2.set(LR.get());
        aarch64_cpu::asm::eret();
    }
}

pub unsafe fn init_mmu(arg: &CpuOnArg) {
    set_mair();

    // Enable TTBR0 and TTBR1 walks, page size = 4K, vaddr size = 48 bits, paddr size = 40 bits.
    let tcr_flags0 = TCR_EL1::EPD0::EnableTTBR0Walks
        + TCR_EL1::TG0::KiB_4
        + TCR_EL1::SH0::Inner
        + TCR_EL1::ORGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
        + TCR_EL1::IRGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
        + TCR_EL1::T0SZ.val(16);
    let tcr_flags1 = TCR_EL1::EPD1::EnableTTBR1Walks
        + TCR_EL1::TG1::KiB_4
        + TCR_EL1::SH1::Inner
        + TCR_EL1::ORGN1::WriteBack_ReadAlloc_WriteAlloc_Cacheable
        + TCR_EL1::IRGN1::WriteBack_ReadAlloc_WriteAlloc_Cacheable
        + TCR_EL1::T1SZ.val(16);
    TCR_EL1.write(TCR_EL1::IPS::Bits_48 + tcr_flags0 + tcr_flags1);
    barrier::isb(barrier::SY);
    let tb = arg.boot_table.raw() as _;
    // let tb = table.raw() as _;

    TTBR1_EL1.set_baddr(tb);
    TTBR0_EL1.set_baddr(tb);

    // Flush the entire TLB
    flush_tlb(None);

    // Enable the MMU and turn on I-cache and D-cache
    SCTLR_EL1.modify(SCTLR_EL1::M::Enable + SCTLR_EL1::C::Cacheable + SCTLR_EL1::I::Cacheable);
    barrier::isb(barrier::SY);
    // unsafe {
    //     let base = 0xfe660000usize;
    //     let base = 0x9000000usize;
    //     (base as *mut u8).write_volatile(b'A');
    //     (base as *mut u8).write_volatile(b'\r');
    //     (base as *mut u8).write_volatile(b'\n');
    // }
    // unsafe {
    //     let base = 0xfe660000usize;
    //     (base as *mut u8).write_volatile(b'A');
    //     (base as *mut u8).write_volatile(b'\r');
    //     (base as *mut u8).write_volatile(b'\n');
    // }
}
