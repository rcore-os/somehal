use core::panic::PanicInfo;

use linked_list_allocator::LockedHeap;
use crate::dbgln;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    dbgln!("panic");
    loop {}
}

#[global_allocator]
pub static HEAP: LockedHeap = LockedHeap::empty();
