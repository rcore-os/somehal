#![no_std]
#![cfg(target_os = "none")]

use log::{debug, info};
use somehal::BootInfo;

use crate::debug::init_log;

extern crate somehal;

mod debug;
pub mod lang_items;
mod mem;

#[somehal::entry]
fn main(args: &BootInfo) -> ! {
    init_log();

    debug!("boot args: {:?}", args);

    mem::init_this();

    // unsafe {
    //     let a = 2usize as *mut u8;
    //     let b = a.read_volatile();
    //     debug!("a: {a:p}, b: {b}");
    // }

    info!("All tests passed!");
}

#[somehal::irq_handler]
fn irq_handler() {
    debug!("IRQ handler called");
    // Here you can handle the IRQ, for example, by reading a register or clearing an interrupt
}
