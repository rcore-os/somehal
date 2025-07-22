use buddy_system_allocator::LockedHeap;
use log::debug;
use somehal::{MemoryRegionKind, boot_info, mem::phys_to_virt};

pub use somehal::mem::*;

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap<32> = LockedHeap::empty();

pub fn init_this() {
    debug!("Initializing memory regions");
    let main_start = boot_info().free_memory_start as usize;
    for ram in boot_info()
        .memory_regions
        .iter()
        .filter(|r| matches!(r.kind, MemoryRegionKind::Ram))
    {
        let start = ram.start;
        let end = ram.end;
        if (start..end).contains(&main_start) {
            let start = phys_to_virt(main_start) as usize;
            let end = phys_to_virt(end) as usize;
            let size = end - start;
            debug!("Initializing heap allocator: start={start:#x}, size={size:#x}");
            unsafe { HEAP_ALLOCATOR.lock().init(start, size) };
            break;
        }
    }
    for ram in boot_info()
        .memory_regions
        .iter()
        .filter(|r| matches!(r.kind, MemoryRegionKind::Ram))
    {
        let start = ram.start;
        let end = ram.end;
        if !(start..end).contains(&main_start) {
            let start = phys_to_virt(main_start) as usize;
            let end = phys_to_virt(end) as usize;
            let size = end - start;
            debug!("Adding to heap allocator: start={start:#x}, size={size:#x}");
            unsafe { HEAP_ALLOCATOR.lock().add_to_heap(start, size) };
        }
    }

    init();
}
