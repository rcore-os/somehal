use kmem::{
    region::{AccessFlags, CacheConfig, MemConfig},
    *,
};
use page_table_generic::*;

use crate::{
    Arch,
    archif::ArchIf,
    config::{PAGE_SIZE, STACK_SIZE},
    dbgln,
};

type Table<'a> = PageTableRef<'a, <Arch as ArchIf>::PageTable>;

#[inline(always)]
fn link_section_end() -> *const u8 {
    unsafe extern "C" {
        fn __boot_stack_bottom();
    }
    __boot_stack_bottom as _
}

fn kernal_kcode_start() -> usize {
    unsafe extern "C" {
        fn __start_BootText();
    }
    __start_BootText as _
}

/// `rsv_space` 在 `boot stack` 之后保留的空间到校
pub fn new_boot_table(mut rsv_space: usize, kcode_offset: usize) -> PhysAddr {
    rsv_space = rsv_space.align_up(PAGE_SIZE);

    dbgln!("Rsv space      : {}", rsv_space);

    let code_end_phys = PhysAddr::from(link_section_end() as usize);

    let start = (code_end_phys + STACK_SIZE + rsv_space).align_up(PAGE_SIZE);

    let mut tmp_alloc = LineAllocator::new(start, GB);

    dbgln!(
        "Tmp Table space: [{}, {})",
        tmp_alloc.iter.raw(),
        tmp_alloc.end.raw()
    );

    let access = &mut tmp_alloc;

    let mut table = early_err!(Table::create_empty(access));
    unsafe {
        let align = 2 * MB;

        let code_start_phys = kernal_kcode_start().align_down(align);
        let code_start = code_start_phys + kcode_offset;
        let code_end: usize = (code_end_phys + kcode_offset).align_up(align).raw();

        let size = (code_end - code_start).max(align);

        dbgln!(
            "code           : [{}, {}) -> [{}, {})",
            code_start,
            code_start + size,
            code_start_phys,
            code_start_phys + size
        );

        early_err!(table.map(
            MapConfig {
                vaddr: code_start.into(),
                paddr: code_start_phys.into(),
                size,
                pte: Arch::new_pte_with_config(MemConfig {
                    access: AccessFlags::Read | AccessFlags::Write | AccessFlags::Execute,
                    cache: CacheConfig::Normal
                }),
                allow_huge: true,
                flush: false,
            },
            access,
        ));

        let size = table.entry_size() * 12;

        dbgln!("eq             : [{}, {})", 0usize, size);
        early_err!(table.map(
            MapConfig {
                vaddr: 0.into(),
                paddr: 0usize.into(),
                size,
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

    dbgln!("Table size     : {}", tmp_alloc.used());

    table.paddr()
}

pub struct LineAllocator {
    pub start: PhysAddr,
    iter: PhysAddr,
    pub end: PhysAddr,
}

impl LineAllocator {
    pub fn new(start: PhysAddr, size: usize) -> Self {
        Self {
            start,
            iter: start,
            end: start + size,
        }
    }

    pub fn alloc(&mut self, layout: core::alloc::Layout) -> Option<PhysAddr> {
        let start = self.iter.align_up(layout.align());
        if start + layout.size() > self.end {
            return None;
        }

        self.iter += layout.size().align_up(layout.align());

        Some(start)
    }

    pub fn used(&self) -> usize {
        self.iter - self.start
    }
}

impl Access for LineAllocator {
    unsafe fn alloc(&mut self, layout: core::alloc::Layout) -> Option<PhysAddr> {
        LineAllocator::alloc(self, layout)
    }

    unsafe fn dealloc(&mut self, _ptr: PhysAddr, _layout: core::alloc::Layout) {}

    fn phys_to_mut(&self, phys: PhysAddr) -> *mut u8 {
        phys.raw() as _
    }
}
