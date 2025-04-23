use crate::paging::*;
use kmem_region::region::MemConfig;

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct Pte(usize);

impl PTEGeneric for Pte {
    fn valid(&self) -> bool {
        todo!()
    }

    fn paddr(&self) -> PhysAddr {
        todo!()
    }

    fn set_paddr(&mut self, _paddr: PhysAddr) {
        todo!()
    }

    fn set_valid(&mut self, _valid: bool) {
        todo!()
    }

    fn is_huge(&self) -> bool {
        todo!()
    }

    fn set_is_huge(&mut self, _b: bool) {
        todo!()
    }
}

impl core::fmt::Debug for Pte {
    fn fmt(&self, _f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        todo!()
    }
}

#[derive(Clone, Copy)]
pub struct Table;

impl TableGeneric for Table {
    type PTE = Pte;
    fn flush(_vaddr: Option<crate::paging::VirtAddr>) {}
}

pub fn new_pte_with_config(_config: MemConfig) -> Pte {
    todo!()
}
