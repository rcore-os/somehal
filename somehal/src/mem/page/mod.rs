use core::alloc::Layout;
use core::panic;

use kmem_region::allocator::LineAllocator;
use page_table_generic::*;

use crate::arch::Arch;
use crate::archif::ArchIf;
use crate::mem::{MEM_REGIONS, init_heap};
use crate::{handle_err, printkv, println};

pub type Table<'a> = PageTableRef<'a, <Arch as ArchIf>::PageTable>;

pub const fn page_size() -> usize {
    <Arch as ArchIf>::PageTable::PAGE_SIZE
}

pub const fn page_levels() -> usize {
    <Arch as ArchIf>::PageTable::LEVEL
}

pub fn page_level_size(level: usize) -> usize {
    page_size() * <Arch as ArchIf>::PageTable::TABLE_LEN.pow((level - 1) as _)
}

pub const fn page_valid_bits() -> usize {
    <Arch as ArchIf>::PageTable::VALID_BITS
}

pub const fn page_valid_addr_mask() -> usize {
    (1 << page_valid_bits()) - 1
}

pub fn new_mapped_table() -> kmem_region::PhysAddr {
    let tmp_size = 8 * MB;

    let start = if let Some(h) =
        unsafe { super::heap::alloc(Layout::from_size_align(tmp_size, page_size()).unwrap()) }
    {
        h
    } else {
        println!("Failed to allocate tmp page table");
        panic!();
    };

    let start = start.as_ptr() as usize;
    let mut tmp_alloc = PageTableAccess(LineAllocator::new(start.into(), tmp_size));

    printkv!(
        "Tmp page allocator",
        "[{:?}, {:?})",
        tmp_alloc.0.start,
        tmp_alloc.0.end
    );

    let access = &mut tmp_alloc;

    let mut table = handle_err!(Table::create_empty(access));

    for region in MEM_REGIONS.clone() {
        unsafe {
            handle_err!(table.map(
                MapConfig {
                    vaddr: region.virt_start.raw().into(),
                    paddr: region.phys_start.raw().into(),
                    size: region.size,
                    pte: Arch::new_pte_with_config(region.config),
                    allow_huge: true,
                    flush: false,
                },
                access,
            ));
        }
    }

    println!("Table size {:#x}", tmp_alloc.0.used());

    table.paddr().raw().into()
}

struct PageTableAccess(LineAllocator);

impl Access for PageTableAccess {
    #[inline(always)]
    unsafe fn alloc(&mut self, layout: core::alloc::Layout) -> Option<PhysAddr> {
        self.0.alloc(layout).map(|p| p.raw().into())
    }

    #[inline(always)]
    unsafe fn dealloc(&mut self, _ptr: PhysAddr, _layout: core::alloc::Layout) {}

    #[inline(always)]
    fn phys_to_mut(&self, phys: PhysAddr) -> *mut u8 {
        phys.raw() as _
    }
}
