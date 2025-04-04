use core::arch::asm;

use kmem::VirtAddr;

#[link_boot::link_boot]
mod _m {
    use core::fmt::Debug;

    use aarch64_cpu::registers::*;
    use kmem::{
        PhysAddr,
        paging::{PTEGeneric, TableGeneric},
        space::AccessFlags,
    };

    pub fn set_kernel_table(addr: PhysAddr) {
        TTBR1_EL1.set_baddr(addr.raw() as _);
        flush_tlb(None);
    }

    pub fn get_kernel_table() -> PhysAddr {
        (TTBR1_EL1.get_baddr() as usize).into()
    }

    pub fn set_user_table(addr: PhysAddr) {
        TTBR0_EL1.set_baddr(addr.raw() as _);
        flush_tlb(None);
    }

    pub fn get_user_table() -> PhysAddr {
        (TTBR0_EL1.get_baddr() as usize).into()
    }

    pub fn flush_tlb(vaddr: Option<VirtAddr>) {
        match vaddr {
            Some(addr) => {
                unsafe { asm!("tlbi vaae1is, {}; dsb nsh; isb", in(reg) addr.raw()) };
            }
            None => {
                unsafe { asm!("tlbi vmalle1is; dsb nsh; isb") };
            }
        }
    }

    pub fn setup_regs() {
        // Device-nGnRnE
        let attr0 = MAIR_EL1::Attr0_Device::nonGathering_nonReordering_noEarlyWriteAck;
        // Normal
        let attr1 = MAIR_EL1::Attr1_Normal_Inner::WriteBack_NonTransient_ReadWriteAlloc
            + MAIR_EL1::Attr1_Normal_Outer::WriteBack_NonTransient_ReadWriteAlloc;
        // WriteThrough
        let attr2 = MAIR_EL1::Attr2_Normal_Inner::WriteThrough_Transient_WriteAlloc
            + MAIR_EL1::Attr2_Normal_Outer::WriteThrough_Transient_WriteAlloc;

        MAIR_EL1.write(attr0 + attr1 + attr2);

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

        Table::flush(None);
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

        fn as_flags(&self) -> PteFlags {
            PteFlags::from_bits_truncate(self.0)
        }

        fn set_mair_idx(&mut self, idx: usize) {
            self.0 &= !Self::MAIR_MASK;
            self.0 |= idx << 2;
        }
    }

    impl PTEGeneric for Pte {
        fn valid(&self) -> bool {
            self.as_flags().contains(PteFlags::VALID)
        }

        fn paddr(&self) -> PhysAddr {
            (self.0 & Self::PHYS_ADDR_MASK).into()
        }

        fn set_paddr(&mut self, paddr: PhysAddr) {
            self.0 &= !Self::PHYS_ADDR_MASK;
            self.0 |= paddr.raw() & Self::PHYS_ADDR_MASK;
        }

        fn set_valid(&mut self, valid: bool) {
            if valid {
                self.0 |= PteFlags::VALID.bits();
            } else {
                self.0 &= !PteFlags::VALID.bits();
            }
        }

        fn is_block(&self) -> bool {
            !self.as_flags().contains(PteFlags::NON_BLOCK)
        }

        fn set_is_block(&mut self, is_block: bool) {
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

    pub fn new_pte_with_config(config: kmem::space::MemConfig) -> Pte {
        let mut flags = PteFlags::AF | PteFlags::VALID;

        if !config.access.contains(AccessFlags::Write) {
            flags |= PteFlags::AP_RO;
        }

        if !config.access.contains(AccessFlags::Execute) {
            flags |= PteFlags::PXN;
        }

        if config.access.contains(AccessFlags::LowerRead) {
            flags |= PteFlags::AP_EL0;
        }

        if !config.access.contains(AccessFlags::LowerExecute) {
            flags |= PteFlags::UXN;
        }

        let mut pte = Pte(flags.bits());

        pte.set_mair_idx(match config.cache {
            kmem::space::CacheConfig::Device => 0,
            kmem::space::CacheConfig::Normal => 1,
            kmem::space::CacheConfig::WriteThrough => 2,
        });

        pte
    }
}
