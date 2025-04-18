use core::arch::asm;

use kmem::{
    VirtAddr,
    region::{ADDR_BITS, AccessFlags, PAGE_LEVELS},
};
use page_table_generic::*;

use somehal_macros::dbgln;

use crate::mem::new_boot_table;

#[inline(always)]
fn flush_tlb(vaddr: Option<kmem::VirtAddr>) {
    if let Some(vaddr) = vaddr {
    } else {
    }
}
#[inline(always)]
fn set_page_table(addr: PhysAddr) {
    flush_tlb(None);
}

pub fn enable_mmu(hartid: usize, fdt: *mut u8, kcode_offset: usize) -> ! {
    unsafe {
        let table = new_boot_table(0, kcode_offset);

        // let entry = entry_vma();
        asm!("", options(nostack, noreturn))
    }
}

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
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct Pte(usize);

impl Pte {
    const PHYS_ADDR_MASK: usize = (1 << 54) - (1 << 10);
}

impl PTEGeneric for Pte {
    fn valid(&self) -> bool {
        PTEFlags::from_bits_truncate(self.0).contains(PTEFlags::V)
    }

    fn paddr(&self) -> kmem::PhysAddr {
        ((self.0 & Self::PHYS_ADDR_MASK) << 2).into()
    }

    fn set_paddr(&mut self, paddr: kmem::PhysAddr) {
        self.0 = (self.0 & !Self::PHYS_ADDR_MASK) | ((paddr.raw() >> 2) & Self::PHYS_ADDR_MASK);
    }

    fn set_valid(&mut self, valid: bool) {
        if valid {
            self.0 |= PTEFlags::V.bits();
        } else {
            self.0 &= !PTEFlags::V.bits();
        }
    }

    fn is_huge(&self) -> bool {
        PTEFlags::from_bits_truncate(self.0).intersects(PTEFlags::R | PTEFlags::W | PTEFlags::X)
    }

    fn set_is_huge(&mut self, b: bool) {
        if !b {
            self.0 &= !((1 << 10) - 1);
            self.0 |= 1;
        }
    }
}

impl core::fmt::Debug for Pte {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("Pte").field(&self.0).finish()
    }
}

#[derive(Clone, Copy)]
pub struct Table;

impl TableGeneric for Table {
    type PTE = Pte;
    const LEVEL: usize = PAGE_LEVELS;
    const VALID_BITS: usize = ADDR_BITS;

    const MAX_BLOCK_LEVEL: usize = 3;
    fn flush(vaddr: Option<VirtAddr>) {
        flush_tlb(vaddr);
    }
}

pub fn new_pte_with_config(config: kmem::region::MemConfig) -> Pte {
    let mut flags = PTEFlags::V | PTEFlags::D | PTEFlags::A | PTEFlags::R | PTEFlags::G;

    if config.access.contains(AccessFlags::Write) {
        flags |= PTEFlags::W;
    }

    if config.access.contains(AccessFlags::Execute) {
        flags |= PTEFlags::X;
    }

    if config.access.contains(AccessFlags::LowerRead) {
        flags |= PTEFlags::U;
    }

    if config.access.contains(AccessFlags::LowerWrite) {
        flags |= PTEFlags::U | PTEFlags::W;
    }

    if config.access.contains(AccessFlags::LowerExecute) {
        flags |= PTEFlags::U | PTEFlags::X;
    }

    Pte(flags.bits())
}
