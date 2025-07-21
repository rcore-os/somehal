use core::panic::PanicInfo;

use aarch64_cpu::asm::wfi;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info.message());
    loop {
        wfi();
    }
}
