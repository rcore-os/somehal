use core::fmt::Debug;

use kmem::{
    VirtAddr,
    paging::{PTEGeneric, TableGeneric},
    region::AccessFlags,
};

bitflags::bitflags! {
    /// Page-table entry flags.
    #[derive(Debug)]
    pub struct PTEFlags: usize {
        /// Whether the PTE is valid.
        const V =   1 << 0;
        /// Whether the page is readable.
        const R =   1 << 1;
        /// Whether the page is writable.
        const W =   1 << 2;
        /// Whether the page is executable.
        const X =   1 << 3;
        /// Whether the page is accessible to user mode.
        const U =   1 << 4;
        /// Designates a global mapping.
        const G =   1 << 5;
        /// Indicates the virtual page has been read, written, or fetched from
        /// since the last time the A bit was cleared.
        const A =   1 << 6;
        /// Indicates the virtual page has been written since the last time the
        /// D bit was cleared.
        const D =   1 << 7;
    }
}
#[link_boot::link_boot]
mod _m {

    #[repr(transparent)]
    #[derive(Clone, Copy)]
    pub struct Pte(usize);

    impl PTEGeneric for Pte {
        fn valid(&self) -> bool {
            PTEFlags::from_bits_truncate(self.0).contains(PTEFlags::V)
        }

        fn paddr(&self) -> kmem::PhysAddr {
            (self.0 >> 10 & ((1usize << 44) - 1)).into()
        }

        fn set_paddr(&mut self, paddr: kmem::PhysAddr) {
            self.0 = (self.0 & !((1usize << 44) - 1)) | (paddr.raw() << 10);
        }

        fn set_valid(&mut self, valid: bool) {
            if valid {
                self.0 |= PTEFlags::V.bits();
            } else {
                self.0 &= !PTEFlags::V.bits();
            }
        }

        fn is_block(&self) -> bool {
            PTEFlags::from_bits_truncate(self.0).intersects(PTEFlags::R | PTEFlags::W | PTEFlags::X)
        }

        fn set_is_block(&mut self, is_block: bool) {
            if is_block {
                self.0 &= !(PTEFlags::R | PTEFlags::W | PTEFlags::X).bits();
            } else {
                self.0 |= PTEFlags::R.bits()
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
        const LEVEL: usize = 3;
        const MAX_BLOCK_LEVEL: usize = 2;

        fn flush(_vaddr: Option<VirtAddr>) {
            riscv::asm::sfence_vma_all();
        }
    }

    pub fn new_pte_with_config(config: kmem::region::MemConfig) -> Pte {
        let mut flags = PTEFlags::V | PTEFlags::D | PTEFlags::A;

        if !config.access.contains(AccessFlags::Write) {
            flags |= PTEFlags::W;
        }

        if !config.access.contains(AccessFlags::Execute) {
            flags |= PTEFlags::X;
        }

        if config.access.contains(AccessFlags::LowerRead) {
            flags |= PTEFlags::U;
        }

        if !config.access.contains(AccessFlags::LowerExecute) {
            flags |= PTEFlags::U;
        }

        Pte(flags.bits())
    }
}
