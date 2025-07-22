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

    info!("All tests passed!");
}
