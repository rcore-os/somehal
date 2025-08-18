use aarch64_cpu::registers::*;
use core::arch::{asm, global_asm};
use kasm_aarch64::aarch64_trap_handler;
use log::*;

use super::context::Context;

#[aarch64_trap_handler(kind = "irq")]
fn handle_irq(_ctx: &Context) {}

#[aarch64_trap_handler(kind = "fiq")]
fn handle_fiq(_ctx: &Context) {}

#[aarch64_trap_handler(kind = "sync")]
fn handle_sync(ctx: &Context) {
    let esr = ESR_EL1.extract();
    let iss = esr.read(ESR_EL1::ISS);
    let elr = ctx.pc;

    if let Some(code) = esr.read_as_enum(ESR_EL1::EC) {
        match code {
            ESR_EL1::EC::Value::SVC64 => {
                warn!("No syscall is supported currently!");
            }
            ESR_EL1::EC::Value::DataAbortLowerEL => handle_data_abort(iss, true),
            ESR_EL1::EC::Value::DataAbortCurrentEL => handle_data_abort(iss, false),
            ESR_EL1::EC::Value::Brk64 => {
                // debug!("BRK #{:#x} @ {:#x} ", iss, tf.elr);
                // tf.elr += 4;
            }
            _ => {
                panic!(
                    "\r\n{:?}\r\nUnhandled synchronous exception @ {:p}: ESR={:#x} (EC {:#08b}, ISS {:#x})",
                    ctx,
                    elr,
                    esr.get(),
                    esr.read(ESR_EL1::EC),
                    esr.read(ESR_EL1::ISS),
                );
            }
        }
    }
}

#[aarch64_trap_handler(kind = "serror")]
fn handle_serror(ctx: &Context) {
    error!("SError exception:");
    let esr = ESR_EL1.extract();
    let _iss = esr.read(ESR_EL1::ISS);
    let elr = ELR_EL1.get();
    error!("{:?}", ctx);
    panic!(
        "Unhandled serror @ {:#x}: ESR={:#x} (EC {:#08b}, ISS {:#x})",
        elr,
        esr.get(),
        esr.read(ESR_EL1::EC),
        esr.read(ESR_EL1::ISS),
    );
}

fn handle_data_abort(iss: u64, _is_user: bool) {
    let wnr = (iss & (1 << 6)) != 0; // WnR: Write not Read
    let cm = (iss & (1 << 8)) != 0; // CM: Cache maintenance
    let reason = if wnr & !cm {
        PageFaultReason::Write
    } else {
        PageFaultReason::Read
    };
    let vaddr = FAR_EL1.get() as usize;

    handle_page_fault(vaddr, reason);
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
    let addr: usize;
    unsafe {
        asm!(
            "adr {0}, __vector_table",
            out(reg) addr,
        );
    }

    match CurrentEL.read(CurrentEL::EL) {
        1 => unsafe {
            asm!("msr vbar_el1, {0}", in(reg) addr);
        },
        2 => unsafe {
            asm!("msr vbar_el2, {0}", in(reg) addr);
        },
        _ => panic!("Unsupported exception level for vector table setup"),
    }
}
