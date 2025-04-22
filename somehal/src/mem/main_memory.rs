use core::{
    alloc::{AllocError, Layout},
    cell::UnsafeCell,
    ptr::NonNull,
};

use kmem::{
    IntAlign, PhysAddr,
    alloc::LineAllocator,
    region::{MemConfig, MemRegion, MemRegionKind},
};
use spin::Mutex;

use super::{MEMORY_MAIN, PhysMemory};
use crate::println;

pub unsafe fn init(start: PhysAddr, end: PhysAddr) {
    if MEMORY_MAIN.is_init() {
        return;
    }
    unsafe {
        MEMORY_MAIN.init(PhysMemory {
            addr: start,
            size: end - start,
        })
    };
}

pub struct RegionAllocator {
    allocator: LineAllocator,
    name: &'static str,
    config: MemConfig,
    kind: MemRegionKind,
    va_offset: usize,
}

impl RegionAllocator {
    pub fn new(
        name: &'static str,
        config: MemConfig,
        kind: MemRegionKind,
        va_offset: usize,
    ) -> Self {
        Self {
            allocator: LineAllocator::new(MEMORY_MAIN.addr, MEMORY_MAIN.size),
            name,
            config,
            kind,
            va_offset,
        }
    }

    pub fn allocate(&mut self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        unsafe {
            let ptr = self.allocator.alloc(layout).ok_or(AllocError {})?;

            Ok(NonNull::slice_from_raw_parts(
                NonNull::new_unchecked(ptr.raw() as _),
                layout.size(),
            ))
        }
    }
}

impl From<RegionAllocator> for MemRegion {
    fn from(value: RegionAllocator) -> Self {
        let al = &value.allocator;

        let m = unsafe { &mut *MEMORY_MAIN.get() }.as_mut().unwrap();
        assert_eq!(m.addr, al.start, "parrel `RegionAllocator` allocator");

        m.addr = al.highest_address();
        m.size -= al.used();

        MemRegion {
            name: value.name,
            phys_start: al.start,
            size: al.used(),
            config: value.config,
            kind: value.kind,
            virt_start: (al.start.raw() + value.va_offset).into(),
        }
    }
}

pub fn alloc(layout: Layout) -> PhysAddr {
    unsafe {
        let end = MEMORY_MAIN.addr + MEMORY_MAIN.size;
        let ptr = MEMORY_MAIN.addr.align_up(layout.align());
        let start = ptr + layout.size();
        let size = end.raw() - start.raw();
        (*MEMORY_MAIN.get()).replace(PhysMemory { addr: start, size });
        ptr
    }
}
