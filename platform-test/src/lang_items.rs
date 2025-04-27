use core::panic::PanicInfo;

use linked_list_allocator::LockedHeap;
use somehal::{
    mem::{memory_regions, region::MemRegionKind},
    println,
};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{info:?}");
    loop {}
}

#[global_allocator]
pub static HEAP: LockedHeap = LockedHeap::empty();

pub fn init_heap() {
    unsafe {
        let mut g = HEAP.lock();

        for region in memory_regions() {
            if matches!(region.kind, MemRegionKind::Memory) {
                println!(
                    "init heap [{:?}, {:?}",
                    region.virt_start,
                    region.virt_start + region.size
                );
                g.init(region.virt_start.as_ptr(), region.size);
                break;
            }
        }
    }
}
