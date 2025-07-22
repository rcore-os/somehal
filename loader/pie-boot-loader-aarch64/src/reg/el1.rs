use aarch64_cpu::{asm::*, registers::*};
use aarch64_cpu_ext::asm::tlb::{VMALLE1, tlbi};

#[inline(always)]
pub fn set_table(addr: usize) {
    TTBR1_EL1.set_baddr(addr as _);
    TTBR0_EL1.set_baddr(addr as _);
}

#[inline(always)]
pub fn setup_sctlr() {
    SCTLR_EL1.modify(SCTLR_EL1::M::Enable + SCTLR_EL1::C::Cacheable + SCTLR_EL1::I::Cacheable);
    barrier::dsb(barrier::SY);
    barrier::isb(barrier::SY);
}

#[inline(always)]
pub fn setup_table_regs() {
    // Device-nGnRE
    let attr0 = MAIR_EL1::Attr0_Device::nonGathering_nonReordering_EarlyWriteAck;
    // Normal
    let attr1 = MAIR_EL1::Attr1_Normal_Inner::WriteBack_NonTransient_ReadWriteAlloc
        + MAIR_EL1::Attr1_Normal_Outer::WriteBack_NonTransient_ReadWriteAlloc;
    // No cache
    let attr2 =
        MAIR_EL1::Attr2_Normal_Inner::NonCacheable + MAIR_EL1::Attr2_Normal_Outer::NonCacheable;
    // WriteThrough
    let attr3 = MAIR_EL1::Attr3_Normal_Inner::WriteThrough_Transient_WriteAlloc
        + MAIR_EL1::Attr3_Normal_Outer::WriteThrough_Transient_WriteAlloc;

    MAIR_EL1.write(attr0 + attr1 + attr2 + attr3);

    // Enable TTBR0 and TTBR1 walks, page size = 4K, vaddr size = 48 bits, paddr size = 40 bits.
    const VADDR_SIZE: u64 = 48;
    const T0SZ: u64 = 64 - VADDR_SIZE;

    let tcr_flags0 = TCR_EL1::EPD0::EnableTTBR0Walks
        + TCR_EL1::TG0::KiB_4
        + TCR_EL1::SH0::Inner
        + TCR_EL1::ORGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
        + TCR_EL1::IRGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
        + TCR_EL1::T0SZ.val(T0SZ);
    let tcr_flags1 = TCR_EL1::EPD1::EnableTTBR1Walks
        + TCR_EL1::TG1::KiB_4
        + TCR_EL1::SH1::Inner
        + TCR_EL1::ORGN1::WriteBack_ReadAlloc_WriteAlloc_Cacheable
        + TCR_EL1::IRGN1::WriteBack_ReadAlloc_WriteAlloc_Cacheable
        + TCR_EL1::T1SZ.val(T0SZ);
    TCR_EL1.write(TCR_EL1::IPS::Bits_48 + tcr_flags0 + tcr_flags1);

    tlbi(VMALLE1);
    barrier::dsb(barrier::SY);
    barrier::isb(barrier::SY);
}
