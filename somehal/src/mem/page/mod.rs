use page_table_generic::*;

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
