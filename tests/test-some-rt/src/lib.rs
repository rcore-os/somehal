#![no_std]

use log::debug;
use some_rt::BootInfo;

use crate::debug::init_log;

extern crate some_rt;

mod debug;
pub mod lang_items;

#[some_rt::entry]
fn main(boot_info: BootInfo) -> ! {
    clean_bss();
    init_log();

    debug!("boot_info: {:?}", boot_info.cpu_id);
}

fn clean_bss() {
    unsafe extern "C" {
        fn _sbss();
        fn _ebss();
    }
    unsafe {
        let bss =
            core::slice::from_raw_parts_mut(_sbss as *mut u8, _ebss as usize - _sbss as usize);
        bss.fill(0);
    }
}
