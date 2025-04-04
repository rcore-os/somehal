use kmem::paging::*;
use kmem::region::MemConfig;

use crate::arch::Arch;
use crate::archif::ArchIf;
use crate::mem::MEM_REGIONS;
use crate::{handle_err, println};

use super::MEMORY_MAIN;
use super::boot::LineAllocator;

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

pub fn new_mapped_table() -> PhysAddr {
    let start = MEMORY_MAIN.addr + MEMORY_MAIN.size / 2;
    let mut tmp_alloc = LineAllocator::new(start, MEMORY_MAIN.size / 2);

    println!(
        "Tmp page allocator: [{:?}, {:?})",
        tmp_alloc.start, tmp_alloc.end
    );

    let access = &mut tmp_alloc;

    let mut table = handle_err!(Table::create_empty(access));

    for region in MEM_REGIONS.clone() {
        unsafe {
            handle_err!(table.map(
                MapConfig {
                    vaddr: region.virt_start,
                    paddr: region.phys_start,
                    size: region.size,
                    pte: Arch::new_pte_with_config(region.config),
                    allow_huge: true,
                    flush: false,
                },
                access,
            ));
        }
    }

    println!("Table size {:#x}", tmp_alloc.used().size);

    table.paddr()
}
