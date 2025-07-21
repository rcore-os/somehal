use core::arch::asm;

use crate::{
    mmu::CacheKind,
    paging::{PTEGeneric, PhysAddr, TableGeneric, VirtAddr},
};
use aarch64_cpu::{asm::*, registers::*};
use aarch64_cpu_ext::asm::tlb::{ALLE2, VAE2IS, tlbi};

pub fn switch_to_elx(bootargs: usize) {
    SPSel.write(SPSel::SP::ELx);
    SP_EL0.set(0);
    let current_el = CurrentEL.read(CurrentEL::EL);
    let ret = sym_lma!(crate::_start);
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

#[inline(always)]
fn flush_tlb(vaddr: Option<VirtAddr>) {
    match vaddr {
        Some(addr) => {
            tlbi(VAE2IS::new(0, addr.raw()));
        }
        None => {
            tlbi(ALLE2);
        }
    }
    barrier::dsb(barrier::SY);
    barrier::isb(barrier::SY);
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

    pub fn new(cache: CacheKind) -> Self {
        let mut flags = PteFlags::empty() | PteFlags::AF | PteFlags::VALID | PteFlags::NON_BLOCK;

        let idx = match cache {
            CacheKind::Device => 0,
            CacheKind::Normal => {
                flags |= PteFlags::INNER | PteFlags::SHAREABLE;
                1
            }
            CacheKind::NoCache => {
                flags |= PteFlags::SHAREABLE;
                2
            }
        };

        let mut s = Self(flags.bits());
        s.set_mair_idx(idx);
        s
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
            self.0 |= (PteFlags::empty() | PteFlags::VALID).bits();
        } else {
            self.0 &= !(PteFlags::empty() | PteFlags::VALID).bits();
        }
    }

    #[inline(always)]
    fn is_huge(&self) -> bool {
        !self.as_flags().contains(PteFlags::NON_BLOCK)
    }

    #[inline(always)]
    fn set_is_huge(&mut self, is_block: bool) {
        let bits = (PteFlags::empty() | PteFlags::NON_BLOCK).bits();
        if is_block {
            self.0 &= !bits;
        } else {
            self.0 |= bits;
        }
    }
}

impl core::fmt::Debug for Pte {
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
