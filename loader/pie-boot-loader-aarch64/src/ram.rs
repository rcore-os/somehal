use core::{alloc::Layout, cell::UnsafeCell};

use crate::{
    OFFSET,
    paging::{Access, PhysAddr},
};
use num_align::NumAlign;
use pie_boot_if::{MemoryRegion, MemoryRegionKind};

struct SimpleAllocator {
    start: usize,
    current: usize, // 当前分配位置
}

impl SimpleAllocator {
    const fn new() -> Self {
        SimpleAllocator {
            start: 0,
            current: 0,
        }
    }

    unsafe fn init(&mut self, kernel_end: usize) {
        self.start = kernel_end;
        self.current = kernel_end;
    }

    pub fn alloc(&mut self, layout: Layout) -> *mut u8 {
        unsafe {
            let start = self.current.align_up(layout.align()) as *mut u8;
            let end = start.add(layout.size());
            self.current = end as usize;
            start
        }
    }
}

/// 单线程内存分配器
struct Allocator(UnsafeCell<SimpleAllocator>);
unsafe impl Sync for Allocator {}
unsafe impl Send for Allocator {}

static RAM_ALLOC: Allocator = Allocator(UnsafeCell::new(SimpleAllocator::new()));
static mut MEMORY_REGIONS_PTR: *mut MemoryRegion = 0 as _;
static mut MEMORY_REGIONS_LEN: usize = 0;

pub struct Ram;

impl Ram {
    pub fn current(&self) -> *mut u8 {
        unsafe { (*RAM_ALLOC.0.get()).current as _ }
    }
}

impl Access for Ram {
    #[inline(always)]
    unsafe fn alloc(&mut self, layout: Layout) -> Option<PhysAddr> {
        Some(unsafe { ((*RAM_ALLOC.0.get()).alloc(layout) as usize).into() })
    }

    #[inline(always)]
    unsafe fn dealloc(&mut self, _ptr: PhysAddr, _layout: Layout) {}

    #[inline(always)]
    fn phys_to_mut(&self, phys: PhysAddr) -> *mut u8 {
        phys.raw() as _
    }
}

pub fn init(kernel_end: usize) {
    unsafe {
        (*RAM_ALLOC.0.get()).init(kernel_end);
    }
}

/// alloc virt addr
fn alloc_with_layout(layout: Layout) -> *mut u8 {
    (unsafe { Ram {}.alloc(layout).unwrap() + OFFSET }.raw()) as _
}

pub fn alloc_phys(size: usize, align: usize) -> *mut u8 {
    unsafe { alloc_with_layout(Layout::from_size_align(size, align).unwrap()).sub(OFFSET) }
}

fn alloc_region() -> *mut MemoryRegion {
    unsafe {
        let layout = Layout::new::<MemoryRegion>();
        let ptr = alloc_with_layout(layout) as *mut MemoryRegion;
        if MEMORY_REGIONS_PTR.is_null() {
            MEMORY_REGIONS_PTR = ptr;
        }
        MEMORY_REGIONS_LEN += 1;
        ptr
    }
}

pub fn memory_regions() -> &'static mut [MemoryRegion] {
    unsafe {
        let ptr = alloc_region();
        let region = MemoryRegion {
            start: (*RAM_ALLOC.0.get()).start,
            end: Ram {}.current() as _,
            kind: MemoryRegionKind::Bootloader,
        };

        ptr.write(region);

        core::slice::from_raw_parts_mut(MEMORY_REGIONS_PTR, MEMORY_REGIONS_LEN)
    }
}

pub fn current() -> *mut u8 {
    Ram {}.current() as _
}
