use kmem::paging::*;

use crate::arch::Arch;
use crate::archif::ArchIf;

pub mod boot;

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
