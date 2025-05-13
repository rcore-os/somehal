use kmem_region::IntAlign;
use kmem_region::allocator::LineAllocator;
use kmem_region::region::{AccessFlags, CacheConfig, MemConfig, OFFSET_LINER, STACK_SIZE};
use page_table_generic::*;

use crate::arch::Arch;
use crate::archif::ArchIf;
use crate::mem::{MEM_REGIONS, stack_top_phys};
use crate::once_static::OnceStatic;
use crate::platform::CpuIdx;
use crate::{handle_err, printkv, println};

static mut IS_RELOCATED: bool = false;
// 包含线性映射
pub(crate) static BOOT_TABLE1: OnceStatic<PhysAddr> = OnceStatic::new();
pub(crate) static BOOT_TABLE2: OnceStatic<PhysAddr> = OnceStatic::new();

static mut TMP_STACK_ITER: usize = 0;

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

pub fn new_line_table(cpu: CpuIdx) -> kmem_region::PhysAddr {
    let mut tmp_alloc = unsafe {
        let start = stack_top_phys(cpu) - STACK_SIZE;
        let tmp_size = STACK_SIZE;
        PageTableAccess(LineAllocator::new(start, tmp_size))
    };

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

            handle_err!(table.map(
                MapConfig {
                    vaddr: region.phys_start.raw().into(),
                    paddr: region.phys_start.raw().into(),
                    size: region.size,
                    pte: Arch::new_pte_with_config(MemConfig {
                        access: AccessFlags::Read | AccessFlags::Write | AccessFlags::Execute,
                        cache: CacheConfig::Normal
                    }),
                    allow_huge: true,
                    flush: false,
                },
                access,
            ));
        }
    }

    table.paddr().raw().into()
}

pub fn new_mapped_table(is_map_liner: bool) -> kmem_region::PhysAddr {
    let mut tmp_alloc = unsafe {
        if TMP_STACK_ITER == 0 {
            TMP_STACK_ITER = stack_top_phys(CpuIdx::primary()).raw() - STACK_SIZE;
        }

        let start = TMP_STACK_ITER.into();
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
        let pte = Arch::new_pte_with_config(region.config);
        unsafe {
            handle_err!(table.map(
                MapConfig {
                    vaddr: region.virt_start.raw().into(),
                    paddr: region.phys_start.raw().into(),
                    size: region.size,
                    pte,
                    allow_huge: true,
                    flush: false,
                },
                access,
            ));

            if is_map_liner {
                handle_err!(table.map(
                    MapConfig {
                        vaddr: region.phys_start.raw().into(),
                        paddr: region.phys_start.raw().into(),
                        size: region.size,
                        pte: Arch::new_pte_with_config(MemConfig {
                            access: AccessFlags::Read | AccessFlags::Write | AccessFlags::Execute,
                            cache: CacheConfig::Device
                        }),
                        allow_huge: true,
                        flush: false,
                    },
                    access,
                ));
            }
        }
    }
    unsafe {
        if is_map_liner {
            BOOT_TABLE1.init(table.paddr());
            printkv!("BOOT_TABLE1", "{:?}", BOOT_TABLE1.as_ref());
        } else {
            BOOT_TABLE2.init(table.paddr());
            printkv!("BOOT_TABLE2", "{:?}", BOOT_TABLE2.as_ref());
        }
    }
    // if is_line_map_main {
    //     let mut start = super::MEMORY_MAIN_ALL.addr.raw();
    //     let end = (start + super::MEMORY_MAIN_ALL.size).align_up(GB);
    //     start = start.align_down(GB);
    //     start = 0;
    //     let size = end - start + GB;

    //     unsafe {
    //         handle_err!(table.map(
    //             MapConfig {
    //                 vaddr: start.into(),
    //                 paddr: start.into(),
    //                 size,
    //                 pte: Arch::new_pte_with_config(MemConfig {
    //                     access: AccessFlags::Read | AccessFlags::Write | AccessFlags::Execute,
    //                     cache: CacheConfig::Device
    //                 }),
    //                 allow_huge: true,
    //                 flush: false,
    //             },
    //             access,
    //         ));
    //     }
    //     printkv!("map liner", "[{:#x}, {:#x})", start, start + size);
    //     unsafe {
    //         BOOT_TABLE1.init(table.paddr());
    //         printkv!("BOOT_TABLE1", "{:?}", BOOT_TABLE1.as_ref());
    //     }
    // } else {
    //     unsafe {
    //         BOOT_TABLE2.init(table.paddr());
    //         printkv!("BOOT_TABLE2", "{:?}", BOOT_TABLE2.as_ref());
    //     }
    // }

    println!("Table size {:#x}", tmp_alloc.0.used());

    unsafe {
        TMP_STACK_ITER += tmp_alloc.0.used().align_up(page_size());
    }

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
