use kmem::VirtAddr;
use kmem::region::AccessFlags;
use riscv::interrupt::Trap;
use riscv::interrupt::supervisor::{Exception as E, Interrupt as I};
use riscv::register::{scause, stval};
use somehal_macros::dbgln;

use crate::println;

use super::context::TrapFrame;

core::arch::global_asm!(
    include_asm_macros!(),
    include_str!("trap.S"),
    trapframe_size = const core::mem::size_of::<TrapFrame>(),
);

fn handle_breakpoint(sepc: &mut usize) {
    // println!("Exception(Breakpoint) @ {:#x} ", sepc);
    *sepc += 2
}

fn handle_page_fault(tf: &TrapFrame, mut access_flags: AccessFlags, is_user: bool) {
    
    let vaddr = VirtAddr::from(stval::read());
    dbgln!(
        "Unhandled {} Page Fault @ {}, fault_vaddr={}:",
        if is_user { "User" } else { "Supervisor" },
        tf.sepc,
        vaddr.raw(),
  
      
    );
}

#[unsafe(no_mangle)]
fn riscv_trap_handler(tf: &mut TrapFrame, from_user: bool) {
    let scause = scause::read();
    if let Ok(cause) = scause.cause().try_into::<I, E>() {
        match cause {
            Trap::Exception(E::LoadPageFault) => {
                handle_page_fault(tf, AccessFlags::Read, from_user)
            }
            Trap::Exception(E::StorePageFault) => {
                handle_page_fault(tf, AccessFlags::Write, from_user)
            }
            Trap::Exception(E::InstructionPageFault) => {
                handle_page_fault(tf, AccessFlags::Execute, from_user)
            }
            Trap::Exception(E::Breakpoint) => handle_breakpoint(&mut tf.sepc),
            Trap::Interrupt(_) => {}
            _ => {
                panic!("Unhandled trap {:?} @ {:#x}:\n{:#x?}", cause, tf.sepc, tf);
            }
        }
    } else {
        panic!(
            "Unknown trap {:?} @ {:#x}:\n{:#x?}",
            scause.cause(),
            tf.sepc,
            tf
        );
    }
}
