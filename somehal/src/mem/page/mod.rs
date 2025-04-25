use core::alloc::Layout;
use core::panic;

use kmem_region::IntAlign;
use kmem_region::allocator::LineAllocator;
use kmem_region::region::{AccessFlags, MemConfig, OFFSET_LINER};
use page_table_generic::*;

use crate::arch::Arch;
use crate::archif::ArchIf;
use crate::mem::MEM_REGIONS;
use crate::{handle_err, printkv, println};

static mut IS_RELOCATED: bool = false;

pub type Table<'a> = PageTableRef<'a, <Arch as ArchIf>::PageTable>;

#[allow(unused)]
pub(crate) fn set_is_relocated() {
    unsafe {
        IS_RELOCATED = true;
    }
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

pub fn new_test_table() -> kmem_region::PhysAddr {
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

    unsafe {
        handle_err!(table.map(
            MapConfig {
                vaddr: 0xffffe00000200000.into(),
                paddr: 0x200000usize.into(),
                size: GB,
                pte: Arch::new_pte_with_config(MemConfig {
                    access: AccessFlags::Read | AccessFlags::Write | AccessFlags::Execute,
                    cache: kmem_region::region::CacheConfig::Normal
                }),
                allow_huge: true,
                flush: false,
            },
            access,
        ));

        handle_err!(table.map(
            MapConfig {
                vaddr: 0xffffefffffe00000.into(),
                paddr: 0x0000000007a00000usize.into(),
                size: 0x7c00000 - 0x7a00000,
                pte: Arch::new_pte_with_config(MemConfig {
                    access: AccessFlags::Read | AccessFlags::Write | AccessFlags::Execute,
                    cache: kmem_region::region::CacheConfig::Normal
                }),
                allow_huge: true,
                flush: false,
            },
            access,
        ));
        handle_err!(table.map(
            MapConfig {
                vaddr: 0x200000.into(),
                paddr: 0x200000usize.into(),
                size: 256 * MB,
                pte: Arch::new_pte_with_config(MemConfig {
                    access: AccessFlags::Read | AccessFlags::Write | AccessFlags::Execute,
                    cache: kmem_region::region::CacheConfig::Normal
                }),
                allow_huge: true,
                flush: false,
            },
            access,
        ));
        handle_err!(table.map(
            MapConfig {
                vaddr: 0xffff800000200000.into(),
                paddr: 0x200000usize.into(),
                size: 256 * MB,
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

    println!("Table size {:#x}", tmp_alloc.0.used());

    table.paddr().raw().into()
}

pub fn new_test_table2() -> kmem_region::PhysAddr {
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

    unsafe {
        handle_err!(table.map(
            MapConfig {
                vaddr: 0x0.into(),
                paddr: 0x0usize.into(),
                size: 0x100000000,
                pte: Arch::new_pte_with_config(MemConfig {
                    access: AccessFlags::Read | AccessFlags::Write | AccessFlags::Execute,
                    cache: kmem_region::region::CacheConfig::Normal
                }),
                allow_huge: true,
                flush: false,
            },
            access,
        ));
        handle_err!(table.map(
            MapConfig {
                vaddr: 0x0.into(),
                paddr: 0x0usize.into(),
                size: 0x100000000,
                pte: Arch::new_pte_with_config(MemConfig {
                    access: AccessFlags::Read | AccessFlags::Write | AccessFlags::Execute,
                    cache: kmem_region::region::CacheConfig::Normal
                }),
                allow_huge: true,
                flush: false,
            },
            access,
        ));

        handle_err!(table.map(
            MapConfig {
                vaddr: 0xffffefffffe00000.into(),
                paddr: 0x0000000007a00000usize.into(),
                size: 0x7c00000 - 0x7a00000,
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
