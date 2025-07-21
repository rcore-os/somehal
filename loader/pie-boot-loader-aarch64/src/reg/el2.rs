use aarch64_cpu::{asm::barrier, registers::*};

pub fn setup_table_regs() {
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

    // Device-nGnRnE
    let attr0 = MAIR_EL2::Attr0_Device::nonGathering_nonReordering_EarlyWriteAck;
    // Normal
    let attr1 = MAIR_EL2::Attr1_Normal_Inner::WriteBack_NonTransient_ReadWriteAlloc
        + MAIR_EL2::Attr1_Normal_Outer::WriteBack_NonTransient_ReadWriteAlloc;
    // No cache
    let attr2 =
        MAIR_EL2::Attr2_Normal_Inner::NonCacheable + MAIR_EL2::Attr2_Normal_Outer::NonCacheable;
    // WriteThrough
    let attr3 = MAIR_EL2::Attr3_Normal_Inner::WriteThrough_Transient_WriteAlloc
        + MAIR_EL2::Attr3_Normal_Outer::WriteThrough_Transient_WriteAlloc;

    MAIR_EL2.write(attr0 + attr1 + attr2 + attr3);

    // Enable TTBR0 and TTBR1 walks, page size = 4K, vaddr size = 48 bits, paddr size = 40 bits.
    const VADDR_SIZE: u64 = 48;
    const T0SZ: u64 = 64 - VADDR_SIZE;

    // Enable TTBR0 and TTBR1 walks, page size = 4K, vaddr size = 48 bits, paddr size = 40 bits.
    let tcr_flags0 = TCR_EL2::TG0::KiB_4
        + TCR_EL2::SH0::Inner
        + TCR_EL2::ORGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
        + TCR_EL2::IRGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
        + TCR_EL2::T0SZ.val(T0SZ);
    TCR_EL2.write(TCR_EL2::PS::Bits_40 + tcr_flags0);
    barrier::isb(barrier::SY);
}

pub fn set_table(addr: usize) {
    TTBR0_EL2.set_baddr(addr as _);
}

#[inline(always)]
pub fn setup_sctlr() {
    SCTLR_EL2.modify(SCTLR_EL2::M::Enable + SCTLR_EL2::C::Cacheable + SCTLR_EL2::I::Cacheable);
}
