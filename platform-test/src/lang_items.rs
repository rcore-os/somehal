use core::panic::PanicInfo;

use somehal::println;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{info:?}");
    loop {}
}
