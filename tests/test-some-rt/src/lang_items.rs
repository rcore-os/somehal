use core::panic::PanicInfo;

use log::error;
use somehal::power::shutdown;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    error!("Panicked: {info:?}");
    shutdown()
}
