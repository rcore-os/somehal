use core::fmt::Debug;

use kmem_region::region::AccessFlags;
use page_table_generic::{PTEGeneric, PhysAddr, TableGeneric, VirtAddr};
use x86_64::structures::paging::page_table::PageTableFlags;

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct Pte(u64);

impl Pte {
    const PHYS_ADDR_MASK: u64 = 0x000f_ffff_ffff_f000;
}

impl PTEGeneric for Pte {
    fn valid(&self) -> bool {
        PageTableFlags::from_bits_truncate(self.0).contains(PageTableFlags::PRESENT)
    }

    fn paddr(&self) -> PhysAddr {
        (self.0 & Self::PHYS_ADDR_MASK).into()
    }

    fn set_paddr(&mut self, paddr: PhysAddr) {
        self.0 = (self.0 & !Self::PHYS_ADDR_MASK) | (paddr.raw() as u64 & Self::PHYS_ADDR_MASK)
    }

    fn set_valid(&mut self, valid: bool) {
        if valid {
            self.0 |= PageTableFlags::PRESENT.bits();
        } else {
            self.0 &= !PageTableFlags::PRESENT.bits();
        }
    }

    fn is_huge(&self) -> bool {
        PageTableFlags::from_bits_truncate(self.0).contains(PageTableFlags::HUGE_PAGE)
    }

    fn set_is_huge(&mut self, is_block: bool) {
        if is_block {
            self.0 |= PageTableFlags::HUGE_PAGE.bits();
        } else {
            self.0 &= !PageTableFlags::HUGE_PAGE.bits();
        }
    }
}

impl Debug for Pte {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("Pte").field(&self.0).finish()
    }
}

#[derive(Clone, Copy)]
pub struct Table;

impl TableGeneric for Table {
    type PTE = Pte;

    fn flush(vaddr: Option<VirtAddr>) {
        unsafe {
            if let Some(vaddr) = vaddr {
                x86::tlb::flush(vaddr.raw());
            } else {
                x86::tlb::flush_all();
            }
        }
    }
}

pub fn new_pte_with_config(config: kmem_region::region::MemConfig) -> Pte {
    let mut flags = PageTableFlags::PRESENT;

    if config.access.contains(AccessFlags::Write) {
        flags |= PageTableFlags::WRITABLE;
    }

    if !config.access.contains(AccessFlags::Execute) {
        flags |= PageTableFlags::NO_EXECUTE;
    }

    if config.access.contains(AccessFlags::LowerRead) {
        flags |= PageTableFlags::USER_ACCESSIBLE;
    }

    if config.access.contains(AccessFlags::LowerWrite) {
        flags |= PageTableFlags::USER_ACCESSIBLE;
    }

    if config.access.contains(AccessFlags::LowerExecute) {
        flags |= PageTableFlags::USER_ACCESSIBLE;
    }

    match config.cache {
        kmem_region::region::CacheConfig::Normal => {}
        kmem_region::region::CacheConfig::Device => {
            flags |= PageTableFlags::NO_CACHE;
        }
        kmem_region::region::CacheConfig::WriteThrough => {
            flags |= PageTableFlags::WRITE_THROUGH;
        }
    }

    Pte(flags.bits())
}
