use core::{
    alloc::{AllocError, Layout},
    ptr::NonNull,
};

use kmem_region::{
    IntAlign, PhysAddr,
    allocator::LineAllocator,
    region::{MemConfig, MemRegion, MemRegionKind},
};

use crate::mem::page::page_size;

use super::{MEMORY_MAIN, PhysMemory};

pub unsafe fn init(start: PhysAddr, end: PhysAddr) {
    if MEMORY_MAIN.is_init() {
        return;
    }
    unsafe {
        MEMORY_MAIN.set(PhysMemory {
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

        let end = m.addr + m.size;

        let start = al.highest_address().align_up(page_size());

        m.addr = start;
        m.size = end - start;

        MemRegion {
            name: value.name,
            phys_start: al.start,
            size: al.used().align_up(page_size()),
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
