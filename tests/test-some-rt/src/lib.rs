#![no_std]
#![cfg(target_os = "none")]

extern crate alloc;
#[macro_use]
extern crate log;

use core::sync::atomic::{AtomicUsize, Ordering};

use alloc::vec::Vec;
use somehal::{
    BootInfo,
    mem::{cpu_id_list, cpu_stack},
    power::cpu_on,
};
use spin::Mutex;

use crate::debug::init_log;

extern crate somehal;

mod debug;
pub mod lang_items;
mod mem;

static CPU_STATED: AtomicUsize = AtomicUsize::new(1);
static SHARED_DATA: Mutex<usize> = Mutex::new(0);
const SHARED_DATA_WANTED: usize = 12345678;

#[somehal::entry]
fn main(args: &BootInfo) -> ! {
    init_log();

    debug!("boot args: {:?}", args);

    mem::init_this();

    {
        let mut data = SHARED_DATA.lock();
        *data = SHARED_DATA_WANTED;
        debug!("shared data: {}", *data);
    }

    let cpu_ls = cpu_id_list().collect::<Vec<_>>();

    for &cpu_id in &cpu_ls {
        debug!("cpu id: {cpu_id:#x}");
        if cpu_id == args.cpu_id {
            continue;
        }
        let stack = cpu_stack(cpu_id); // Example stack top address for the new CPU

        cpu_on(cpu_id as _, stack.end as _).unwrap();
    }

    while CPU_STATED.load(Ordering::SeqCst) < cpu_ls.len() {
        core::hint::spin_loop();
    }

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
    debug!("Secondary CPU {cpu_id} started");
    {
        let data = SHARED_DATA.lock();
        assert_eq!(*data, SHARED_DATA_WANTED);
        debug!("Secondary CPU {cpu_id} read shared data: {}", *data);
    }
    CPU_STATED.fetch_add(1, Ordering::SeqCst);
    loop {
        core::hint::spin_loop();
    }
}
