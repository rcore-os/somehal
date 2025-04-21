use core::alloc::Layout;

use kmem::IntAlign;
use kmem::region::{MemConfig, MemRegion, MemRegionKind};
use kmem::{PhysAddr, alloc::LineAllocator};

use super::PhysMemory;
use super::{MEMORY_MAIN, mem_region_add};

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

impl Drop for RegionAllocator {
    fn drop(&mut self) {
        let m = unsafe { &mut *MEMORY_MAIN.get() }.as_mut().unwrap();
        m.addr = self.allocator.highest_address();
        m.size -= self.allocator.used();

        let region = MemRegion {
            name: self.name,
            phys_start: self.allocator.start,
            size: self.allocator.used(),
            config: self.config,
            kind: self.kind,
            virt_start: (self.allocator.start.raw() + self.va_offset).into(),
        };

        mem_region_add(region);
    }
}

pub unsafe fn region_allocator(
    name: &'static str,
    config: MemConfig,
    kind: MemRegionKind,
    va_offset: usize,
) -> RegionAllocator {
    RegionAllocator {
        allocator: LineAllocator::new(MEMORY_MAIN.addr, MEMORY_MAIN.size),
        name,
        config,
        kind,
        va_offset,
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
