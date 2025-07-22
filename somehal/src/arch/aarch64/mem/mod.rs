mod mmu;

use core::ptr::NonNull;

pub use crate::common::mem::*;

// After GlobalAlloc is implemented, this will be used as the global allocator.
pub fn init() {
    mmu::init_mmu();
}

pub fn iomap(phys: usize, size: usize) -> Result<NonNull<u8>, page_table_generic::PagingError> {
    let vaddr = phys_to_virt(phys);
    mmu::mmap(MapRangeConfig {
        vaddr,
        paddr: phys,
        size,
        name: "iomap",
        cache: CacheKind::Device,
        access: AccessKind::ReadWrite,
        cpu_share: false,
    })?;

    Ok(NonNull::new(vaddr).unwrap())
}
