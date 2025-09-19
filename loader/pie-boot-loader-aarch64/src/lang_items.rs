use core::panic::PanicInfo;

use aarch64_cpu::asm::wfi;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("panic: {}", info.message());
    loop {
        wfi();
    }
}
