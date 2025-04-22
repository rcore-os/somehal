use core::{cell::UnsafeCell, mem::MaybeUninit, ops::Deref};

use crate::paging::*;
use kmem_region::{
    IntAlign,
    alloc::LineAllocator,
    region::{AccessFlags, CacheConfig, MemConfig},
};

use crate::{Arch, BootInfo, archif::ArchIf, dbgln};

type Table<'a> = PageTableRef<'a, <Arch as ArchIf>::PageTable>;

struct StaticCell<T>(UnsafeCell<T>);

unsafe impl<T> Sync for StaticCell<T> {}
unsafe impl<T> Send for StaticCell<T> {}

impl<T> StaticCell<T> {
    pub const fn new() -> Self {
        let a = MaybeUninit::zeroed();
        let a = unsafe { a.assume_init() };
        Self(UnsafeCell::new(a))
    }
}

static BOOT_INFO: StaticCell<BootInfo> = StaticCell::new();
static PHYS_ALLOCATOR: StaticCell<LineAllocator> = StaticCell::new();
static mut BOOT_TABLE: usize = 0;

impl<T> Deref for StaticCell<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0.get() }
    }
}

pub(crate) fn clean_boot_info() {
    unsafe {
        *BOOT_INFO.0.get() = BootInfo::new();
    }
}

pub(crate) unsafe fn edit_boot_info(f: impl FnOnce(&mut BootInfo)) {
    unsafe {
        let info = &mut *BOOT_INFO.0.get();
        info.main_memory_free_start = PHYS_ALLOCATOR.highest_address();

        f(info);
    }
}

pub(crate) fn boot_info() -> BootInfo {
    unsafe { &*BOOT_INFO.0.get() }.clone()
}

pub(crate) fn boot_info_addr() -> *const BootInfo {
    BOOT_INFO.0.get()
}

pub(crate) fn init_phys_allocator() {
    unsafe {
        *PHYS_ALLOCATOR.0.get() =
            LineAllocator::new(kmem_region::PhysAddr::from(link_section_end() as usize), GB);
    }
}

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

fn table_len() -> usize {
    <<Arch as ArchIf>::PageTable as TableGeneric>::TABLE_LEN
}

/// `rsv_space` 在 `boot stack` 之后保留的空间到校
pub fn new_boot_table(kcode_offset: usize) -> PhysAddr {
    let code_end_phys = PhysAddr::from(link_section_end() as usize);

    let access = unsafe { &mut *PHYS_ALLOCATOR.0.get() };

    dbgln!(
        "Tmp Table space: [{}, {})",
        access.start.raw(),
        access.end.raw()
    );

    let mut table = early_err!(Table::create_empty(access));
    unsafe {
        let align = GB;

        let code_start_phys = kernal_kcode_start().align_down(align);
        let code_start = code_start_phys + kcode_offset;
        let code_end: usize = (code_end_phys + kcode_offset).raw().align_up(align);

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

        let size = if table.entry_size() == table.max_block_size() {
            table.entry_size() * (table_len() / 2)
        } else {
            table.max_block_size() * table_len()
        };

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

    dbgln!(
        "Table size     : {}",
        access.highest_address().raw() - access.start.raw()
    );

    let addr = table.paddr();

    unsafe {
        edit_boot_info(|_f| {});
        BOOT_TABLE = addr.raw();
    }

    addr
}

impl Access for LineAllocator {
    #[inline(always)]
    unsafe fn alloc(&mut self, layout: core::alloc::Layout) -> Option<PhysAddr> {
        LineAllocator::alloc(self, layout).map(|p| p.raw().into())
    }

    #[inline(always)]
    unsafe fn dealloc(&mut self, _ptr: PhysAddr, _layout: core::alloc::Layout) {}

    #[inline(always)]
    fn phys_to_mut(&self, phys: PhysAddr) -> *mut u8 {
        phys.raw() as _
    }
}
