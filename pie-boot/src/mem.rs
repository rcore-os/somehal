use core::{alloc::Layout, cell::UnsafeCell, mem::MaybeUninit, ops::Deref, ptr::NonNull};

use crate::{config::BOOT_STACK_SIZE, paging::*};
use kmem_region::{
    IntAlign,
    allocator::LineAllocator,
    region::{AccessFlags, CacheConfig, MemConfig, PAGE_SIZE},
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

static mut BOOT_TABLE: usize = 0;
static BOOT_INFO: StaticCell<BootInfo> = StaticCell::new();
static PHYS_ALLOCATOR: StaticCell<LineAllocator> = StaticCell::new();
static mut FDT: usize = 0;
static mut FDT_SIZE: usize = 0;

impl<T> Deref for StaticCell<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0.get() }
    }
}

pub unsafe fn clean_bss() {
    unsafe extern "C" {
        fn __start_BootBss();
        fn __stop_BootBss();
    }
    unsafe {
        let start = __start_BootBss as *mut u8;
        let end = __stop_BootBss as *mut u8;
        let len = end as usize - start as usize;
        for i in 0..len {
            start.add(i).write(0);
        }
        (*BOOT_INFO.0.get()) = BootInfo::default();
    }
}
pub(crate) unsafe fn edit_boot_info(f: impl FnOnce(&mut BootInfo)) {
    unsafe {
        let info = &mut *BOOT_INFO.0.get();
        f(info);
    }
}

pub(crate) fn boot_info() -> BootInfo {
    unsafe {
        let info = &mut *BOOT_INFO.0.get();
        early_err!(info.memory_regions.try_push(crate::MemoryRegion {
            start: PHYS_ALLOCATOR.start.raw(),
            end: PHYS_ALLOCATOR.highest_address().raw(),
            kind: crate::MemoryKind::Reserved,
        }));
        info.highest_address = PHYS_ALLOCATOR.highest_address().raw();
        info.fdt = get_fdt_ptr().map(|ptr| (ptr, FDT_SIZE));
        info.clone()
    }
}

pub(crate) fn init_phys_allocator() {
    unsafe {
        *PHYS_ALLOCATOR.0.get() =
            LineAllocator::new(kmem_region::PhysAddr::from(kernel_code_end() as usize), GB);

        reserved_alloc::<[u8; BOOT_STACK_SIZE]>();
    }
}

pub(crate) unsafe fn reserved_alloc_with_layout(layout: Layout) -> Option<NonNull<u8>> {
    unsafe {
        (*PHYS_ALLOCATOR.0.get())
            .alloc(layout)
            .map(|o| NonNull::new_unchecked(o.raw() as _))
    }
}

pub(crate) unsafe fn reserved_alloc<T>() -> Option<NonNull<T>> {
    unsafe { reserved_alloc_with_layout(Layout::new::<T>()).map(|o| o.cast()) }
}

#[inline(always)]
fn kernel_code_end() -> *const u8 {
    unsafe extern "C" {
        fn __kernel_code_end();
    }
    __kernel_code_end as _
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
    let code_end_phys = PhysAddr::from(kernel_code_end() as usize);

    let access = unsafe { &mut *PHYS_ALLOCATOR.0.get() };
    dbgln!("BootTable space: [{}, --)", access.start.raw());

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
        BOOT_TABLE = addr.raw();
        edit_boot_info(|_f| {});
    }

    addr
}
pub(crate) unsafe fn set_fdt_ptr(fdt: *mut u8) {
    unsafe {
        FDT = fdt as _;
    }
}

pub(crate) fn get_fdt_ptr() -> Option<NonNull<u8>> {
    unsafe {
        if FDT == 0 {
            return None;
        }

        NonNull::new(FDT as _)
    }
}

pub(crate) unsafe fn save_fdt() -> Option<()> {
    const FDT_MAGIC: u32 = 0xd00dfeed;
    unsafe {
        let ptr = NonNull::new(FDT as *mut u32)?;
        let magic = u32::from_be(ptr.read());
        if magic != FDT_MAGIC {
            return None;
        }
        let len = u32::from_be(ptr.add(1).read()) as usize;
        let addr = reserved_alloc_with_layout(Layout::from_size_align(len, PAGE_SIZE).unwrap())?;

        let src = core::slice::from_raw_parts(ptr.cast::<u8>().as_ptr(), len);
        let dst = core::slice::from_raw_parts_mut(addr.as_ptr(), len);

        dst.copy_from_slice(src);

        set_fdt_ptr(dst.as_mut_ptr());
        FDT_SIZE = len;
    }
    Some(())
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
