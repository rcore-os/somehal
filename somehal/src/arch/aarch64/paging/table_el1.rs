use core::arch::asm;

use kmem::VirtAddr;

#[link_boot::link_boot]
mod _m {
    use core::fmt::Debug;

    use aarch64_cpu::registers::*;
    use kmem::{PhysAddr, space::AccessFlags};
    use page_table_arm::{PTE, PTEFlags};
    use page_table_generic::{PTEGeneric, TableGeneric};

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

    pub fn get_user_table() -> usize {
        TTBR0_EL1.get_baddr() as _
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
        let attr2 = MAIR_EL1::Attr2_Normal_Inner::WriteThrough_NonTransient_ReadWriteAlloc
            + MAIR_EL1::Attr2_Normal_Outer::WriteThrough_NonTransient_ReadWriteAlloc;

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

    #[repr(transparent)]
    #[derive(Clone, Copy)]
    pub struct Pte(PTE);

    impl PTEGeneric for Pte {
        fn valid(&self) -> bool {
            self.0.get_flags().contains(PTEFlags::VALID)
        }

        fn paddr(&self) -> page_table_generic::PhysAddr {
            self.0.paddr().into()
        }

        fn set_paddr(&mut self, paddr: page_table_generic::PhysAddr) {
            self.0.set_paddr(paddr.raw());
        }

        fn set_valid(&mut self, valid: bool) {
            let mut v = self.0.get_flags();
            if valid {
                v.insert(PTEFlags::VALID);
            } else {
                v.remove(PTEFlags::VALID);
            }
            self.0.set_flags(v);
        }

        fn is_block(&self) -> bool {
            !self.0.get_flags().contains(PTEFlags::NON_BLOCK)
        }

        fn set_is_block(&mut self, is_block: bool) {
            let mut v = self.0.get_flags();
            if is_block {
                v.remove(PTEFlags::NON_BLOCK);
            } else {
                v.insert(PTEFlags::NON_BLOCK);
            }
            self.0.set_flags(v);
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

        fn flush(vaddr: Option<page_table_generic::VirtAddr>) {
            flush_tlb(vaddr.map(|o| o.raw().into()));
        }
    }

    pub fn new_pte_with_config(config: kmem::space::MemConfig) -> Pte {
        let mut pte = PTE::from_paddr(0);
        let mut flags = PTEFlags::AF | PTEFlags::VALID;

        if !config.access.contains(AccessFlags::Write) {
            flags |= PTEFlags::AP_RO;
        }

        if !config.access.contains(AccessFlags::Execute) {
            flags |= PTEFlags::PXN;
        }

        if config.access.contains(AccessFlags::LowerRead) {
            flags |= PTEFlags::AP_EL0;
        }

        if !config.access.contains(AccessFlags::LowerExecute) {
            flags |= PTEFlags::UXN;
        }

        pte.set_flags(flags);
        pte.set_mair_idx(match config.cache {
            kmem::space::CacheConfig::Normal => 1,
            kmem::space::CacheConfig::NoCache => 0,
            kmem::space::CacheConfig::WriteThrough => 2,
        });

        Pte(pte)
    }
}
