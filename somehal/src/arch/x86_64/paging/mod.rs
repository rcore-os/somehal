use core::{arch::asm, fmt::Debug};

use kmem::{paging::{PTEGeneric, TableGeneric}, VirtAddr};

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct Pte(usize);

impl PTEGeneric for Pte {
    fn valid(&self) -> bool {
        todo!()
    }

    fn paddr(&self) -> kmem::PhysAddr {
        todo!()
    }

    fn set_paddr(&mut self, paddr: kmem::PhysAddr) {
        todo!()
    }

    fn set_valid(&mut self, valid: bool) {
        todo!()
    }

    fn is_block(&self) -> bool {
        todo!()
    }

    fn set_is_block(&mut self, is_block: bool) {
        todo!()
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
    }
}
