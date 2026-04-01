use aarch64_cpu::registers::*;
use core::arch::{asm, global_asm};
use kasm_aarch64::aarch64_trap_handler;
use log::*;

use super::context::Context;

#[aarch64_trap_handler(kind = "irq")]
fn handle_irq(_ctx: &Context) {
    unsafe extern "Rust" {
        fn __somehal_handle_irq();
    }
    unsafe {
        __somehal_handle_irq();
    }
}

#[unsafe(no_mangle)]
extern "Rust" fn __somehal_handle_irq_default() {}

#[aarch64_trap_handler(kind = "fiq")]
fn handle_fiq(_ctx: &Context) {}

#[aarch64_trap_handler(kind = "sync")]
fn handle_sync(ctx: &Context) -> usize {
    let (esr, elr, far, current_el) = current_exception_state();
    let iss = esr & 0x01ff_ffff;
    let ec = (esr >> 26) & 0x3f;

    match ec {
        0b01_0101 => {
            warn!("No syscall is supported currently!");
        }
        0b10_0100 => handle_data_abort(iss, true, far),
        0b10_0101 => handle_data_abort(iss, false, far),
        0b11_1100 => {
            // BRK instruction.
        }
        _ => {
            panic!(
                "\r\n{:?}\r\nUnhandled synchronous exception @ {:#x}: EL={} ESR={:#x} (EC {:#08b}, ISS {:#x}) FAR={:#x}",
                ctx, elr, current_el, esr, ec, iss, far,
            );
        }
    }
}

#[aarch64_trap_handler(kind = "serror")]
fn handle_serror(ctx: &Context) -> usize {
    error!("SError exception:");
    let (esr, elr, _far, current_el) = current_exception_state();
    let iss = esr & 0x01ff_ffff;
    let ec = (esr >> 26) & 0x3f;
    error!("{:?}", ctx);
    panic!(
        "Unhandled serror @ {:#x}: EL={} ESR={:#x} (EC {:#08b}, ISS {:#x})",
        elr, current_el, esr, ec, iss,
    );
}

fn handle_data_abort(iss: u64, _is_user: bool, far: usize) {
    let wnr = (iss & (1 << 6)) != 0; // WnR: Write not Read
    let cm = (iss & (1 << 8)) != 0; // CM: Cache maintenance
    let reason = if wnr & !cm {
        PageFaultReason::Write
    } else {
        PageFaultReason::Read
    };
    let vaddr = far;

    handle_page_fault(vaddr, reason);
}

fn current_exception_state() -> (u64, usize, usize, usize) {
    match CurrentEL.read(CurrentEL::EL) {
        1 => (
            ESR_EL1.get(),
            ELR_EL1.get() as usize,
            FAR_EL1.get() as usize,
            1,
        ),
        2 => (
            ESR_EL2.get(),
            ELR_EL2.get() as usize,
            FAR_EL2.get() as usize,
            2,
        ),
        3 => (
            ESR_EL3.get(),
            ELR_EL3.get() as usize,
            FAR_EL3.get() as usize,
            3,
        ),
        el => (0, 0, 0, el as usize),
    }
}

#[derive(Debug)]
pub enum PageFaultReason {
    Read,
    Write,
}

pub fn handle_page_fault(vaddr: usize, reason: PageFaultReason) {
    panic!("Invalid addr fault @{vaddr:#x}, reason: {reason:?}");
}

global_asm!(
    include_str!("vectors.s"),
    irq_handler = sym handle_irq,
    fiq_handler = sym handle_fiq,
    sync_handler = sym handle_sync,
    serror_handler = sym handle_serror,
);

pub fn setup() {
    let el = CurrentEL.read(CurrentEL::EL);
    let _tmp: usize;
    unsafe {
        match el {
            1 => asm!(
                "
        adrp      {0}, vector_table_el1
        add       {0}, {0}, :lo12:vector_table_el1
        MSR      VBAR_EL1, {0}
        ",
                out(reg) _tmp,
            ),
            2 => asm!("
        adrp      {0}, vector_table_el1
        add       {0}, {0}, :lo12:vector_table_el1
        MSR      VBAR_EL2, {0}
            ", out(reg) _tmp),
            _ => {
                panic!("Unsupported EL: {el}");
            }
        }
    }
}
