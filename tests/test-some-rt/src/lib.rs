#![no_std]
#![cfg(target_os = "none")]

use log::{debug, info};
use somehal::{BootInfo, power::cpu_on};

use crate::debug::init_log;

extern crate somehal;

mod debug;
pub mod lang_items;
mod mem;

static CPU_STATED: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);

#[somehal::entry]
fn main(args: &BootInfo) -> ! {
    init_log();

    debug!("boot args: {:?}", args);

    mem::init_this();

    // cpu_on_1();
    // cpu_on_test();
    // debug!("cpu_on_1 returned");
    // while !CPU_STATED.load(core::sync::atomic::Ordering::SeqCst) {
    //     core::hint::spin_loop();
    // }

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

#[somehal::secondary_entry]
fn secondary(cpu_id: usize) {
    // unsafe {
    //     let addr = 0xffff90002800d000usize;
    //     (addr as *mut u8).write_volatile(b'E');
    //     (addr as *mut u8).write_volatile(b'\r');
    //     (addr as *mut u8).write_volatile(b'\n');
    // }

    // debug!("Secondary CPU {cpu_id} started");
    CPU_STATED.store(true, core::sync::atomic::Ordering::SeqCst);
    loop {
        core::hint::spin_loop();
    }
}

/// Power on a CPU
fn cpu_on_1() {
    // let cpu_id = 0x201;
    // let stack_top = 0xf1000000; // Example stack top address for the new CPU

    // let cpu_id = 0x1;
    // let stack_top = 0x47000000;

    let cpu_id = 0x200;
    let stack_top = 0xf0000000;
    cpu_on(cpu_id, stack_top).unwrap();
}
