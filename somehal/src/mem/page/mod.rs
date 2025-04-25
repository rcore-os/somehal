use core::alloc::Layout;
use core::panic;

use kmem_region::IntAlign;
use kmem_region::allocator::LineAllocator;
use kmem_region::region::{AccessFlags, MemConfig, OFFSET_LINER, STACK_SIZE};
use page_table_generic::*;

use crate::arch::Arch;
use crate::archif::ArchIf;
use crate::mem::{MEM_REGIONS, stack_top_cpu0};
use crate::{handle_err, printkv, println};

static mut IS_RELOCATED: bool = false;

pub type Table<'a> = PageTableRef<'a, <Arch as ArchIf>::PageTable>;

pub(crate) fn set_is_relocated() {
    unsafe {
        IS_RELOCATED = true;
    }
}

#[allow(unused)]
pub(crate) fn is_relocated() -> bool {
    unsafe { IS_RELOCATED }
}

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

pub fn new_mapped_table(is_line_map_main: bool) -> kmem_region::PhysAddr {
    let mut tmp_alloc = if is_line_map_main {
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
        PageTableAccess(LineAllocator::new(start.into(), tmp_size))
    } else {
        let start = stack_top_cpu0() - STACK_SIZE;
        let tmp_size = STACK_SIZE;
        PageTableAccess(LineAllocator::new(start, tmp_size))
    };

    printkv!(
        "Tmp page allocator",
        "[{:?}, {:?})",
        tmp_alloc.0.start,
        tmp_alloc.0.end
    );

    let access = &mut tmp_alloc;

    let mut table = handle_err!(Table::create_empty(access));

    for region in MEM_REGIONS.iter() {
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

    if is_line_map_main {
        let mut start = super::MEMORY_MAIN.addr.raw();
        let end = (start + super::MEMORY_MAIN.size).align_up(GB);
        start = start.align_down(GB);
        let size = end - start;

        unsafe {
            handle_err!(table.map(
                MapConfig {
                    vaddr: start.into(),
                    paddr: start.into(),
                    size,
                    pte: Arch::new_pte_with_config(MemConfig {
                        access: AccessFlags::Read | AccessFlags::Write | AccessFlags::Execute,
                        cache: kmem_region::region::CacheConfig::Normal
                    }),
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
        let mut addr = phys.raw();
        if unsafe { IS_RELOCATED } {
            addr += OFFSET_LINER;
        }
        addr as _
    }
}
