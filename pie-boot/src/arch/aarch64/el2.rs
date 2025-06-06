use core::arch::asm;
use core::fmt::Debug;

use aarch64_cpu::{asm::barrier, registers::*};
use kmem_region::region::AccessFlags;

use super::primary_entry;
use crate::paging::{PTEGeneric, PhysAddr, TableGeneric, VirtAddr};

pub fn switch_to_elx(dtb: *mut u8) {
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
        unsafe {
            asm!(
            "adr    x2, {}",
            "mov    x0, {}",
            "msr    elr_el3, x2",
             sym primary_entry,
             in(reg) dtb,
                );
        }
    }
}

#[inline(always)]
pub fn flush_tlb(vaddr: Option<VirtAddr>) {
    match vaddr {
        Some(addr) => unsafe { asm!("tlbi vae2is, {}; dsb sy; isb", in(reg) addr.raw()) },
        None => {
            unsafe { asm!("tlbi alle2is; dsb sy; isb") };
        }
    }
}

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
    let attr0 = MAIR_EL2::Attr0_Device::nonGathering_nonReordering_noEarlyWriteAck;
    // Normal
    let attr1 = MAIR_EL2::Attr1_Normal_Inner::WriteBack_NonTransient_ReadWriteAlloc
        + MAIR_EL2::Attr1_Normal_Outer::WriteBack_NonTransient_ReadWriteAlloc;
    // WriteThrough
    let attr2 = MAIR_EL2::Attr2_Normal_Inner::WriteThrough_Transient_WriteAlloc
        + MAIR_EL2::Attr2_Normal_Outer::WriteThrough_Transient_WriteAlloc;

    MAIR_EL2.write(attr0 + attr1 + attr2);

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

pub fn set_table(addr: PhysAddr) {
    TTBR0_EL2.set_baddr(addr.raw() as _);
}

#[inline(always)]
pub fn setup_sctlr() {
    SCTLR_EL2.modify(SCTLR_EL2::M::Enable + SCTLR_EL2::C::Cacheable + SCTLR_EL2::I::Cacheable);
}

bitflags::bitflags! {
    #[repr(transparent)]
    /// Memory attribute fields in the VMSAv8-64 translation table format descriptors.
    #[derive(Clone, Copy)]
    pub struct PteFlags: usize {
        // Attribute fields in stage 1 VMSAv8-64 Block and Page descriptors:

        /// Whether the descriptor is valid.
        const VALID =       1 << 0;
        /// The descriptor gives the address of the next level of translation table or 4KB page.
        /// (not a 2M, 1G block)
        const NON_BLOCK =   1 << 1;

        /// Non-secure bit. For memory accesses from Secure state, specifies whether the output
        /// address is in Secure or Non-secure memory.
        const NS =          1 << 5;
        /// Access permission: accessable at EL0.
        const AP_EL0 =      1 << 6;
        /// Access permission: read-only.
        const AP_RO =       1 << 7;
        /// Shareability: Inner Shareable (otherwise Outer Shareable).
        const INNER =       1 << 8;
        /// Shareability: Inner or Outer Shareable (otherwise Non-shareable).
        const SHAREABLE =   1 << 9;
        /// The Access flag.
        const AF =          1 << 10;
        /// The not global bit.
        const NG =          1 << 11;
        /// Indicates that 16 adjacent translation table entries point to contiguous memory regions.
        const CONTIGUOUS =  1 <<  52;
        /// The Privileged execute-never field.
        const PXN =         1 <<  53;
        /// The Execute-never or Unprivileged execute-never field.
        const UXN =         1 <<  54;

        // Next-level attributes in stage 1 VMSAv8-64 Table descriptors:

        /// PXN limit for subsequent levels of lookup.
        const PXN_TABLE =           1 << 59;
        /// XN limit for subsequent levels of lookup.
        const XN_TABLE =            1 << 60;
        /// Access permissions limit for subsequent levels of lookup: access at EL0 not permitted.
        const AP_NO_EL0_TABLE =     1 << 61;
        /// Access permissions limit for subsequent levels of lookup: write access not permitted.
        const AP_NO_WRITE_TABLE =   1 << 62;
        /// For memory accesses from Secure state, specifies the Security state for subsequent
        /// levels of lookup.
        const NS_TABLE =            1 << 63;
    }
}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct Pte(usize);

impl Pte {
    const PHYS_ADDR_MASK: usize = 0x0000_ffff_ffff_f000; // bits 12..48
    const MAIR_MASK: usize = 0b111 << 2;

    #[inline(always)]
    fn as_flags(&self) -> PteFlags {
        PteFlags::from_bits_truncate(self.0)
    }

    #[inline(always)]
    fn set_mair_idx(&mut self, idx: usize) {
        self.0 &= !Self::MAIR_MASK;
        self.0 |= idx << 2;
    }
}

impl PTEGeneric for Pte {
    #[inline(always)]
    fn valid(&self) -> bool {
        self.as_flags().contains(PteFlags::VALID)
    }

    #[inline(always)]
    fn paddr(&self) -> PhysAddr {
        (self.0 & Self::PHYS_ADDR_MASK).into()
    }

    #[inline(always)]
    fn set_paddr(&mut self, paddr: PhysAddr) {
        self.0 &= !Self::PHYS_ADDR_MASK;
        self.0 |= paddr.raw() & Self::PHYS_ADDR_MASK;
    }

    #[inline(always)]
    fn set_valid(&mut self, valid: bool) {
        if valid {
            self.0 |= PteFlags::VALID.bits();
        } else {
            self.0 &= !PteFlags::VALID.bits();
        }
    }

    #[inline(always)]
    fn is_huge(&self) -> bool {
        !self.as_flags().contains(PteFlags::NON_BLOCK)
    }

    #[inline(always)]
    fn set_is_huge(&mut self, is_block: bool) {
        if is_block {
            self.0 &= !PteFlags::NON_BLOCK.bits();
        } else {
            self.0 |= PteFlags::NON_BLOCK.bits();
        }
    }
}

impl Debug for Pte {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "PTE {:?}", self.paddr())
    }
}

#[derive(Clone, Copy)]
pub struct Table;

impl TableGeneric for Table {
    type PTE = Pte;

    fn flush(vaddr: Option<VirtAddr>) {
        flush_tlb(vaddr.map(|o| o.raw().into()));
    }
}

pub fn new_pte_with_config(config: kmem_region::region::MemConfig) -> Pte {
    let mut flags = PteFlags::AF | PteFlags::VALID | PteFlags::NON_BLOCK;

    if !config.access.contains(AccessFlags::Write) {
        flags |= PteFlags::AP_RO;
    }

    if !config.access.contains(AccessFlags::Execute) {
        flags |= PteFlags::UXN;
    }

    if config.access.contains(AccessFlags::LowerRead) {
        flags |= PteFlags::AP_EL0;
    }

    let mut pte = Pte(flags.bits());

    pte.set_mair_idx(match config.cache {
        kmem_region::region::CacheConfig::Device => 0,
        kmem_region::region::CacheConfig::Normal => 1,
        kmem_region::region::CacheConfig::WriteThrough => 2,
    });

    pte
}
