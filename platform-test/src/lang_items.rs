use core::panic::PanicInfo;

use linked_list_allocator::LockedHeap;
use somehal::println;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{info:?}");
    loop {}
}



#[global_allocator]
pub static HEAP: LockedHeap = LockedHeap::empty();