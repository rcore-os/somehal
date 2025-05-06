use core::arch::asm;

use aarch64_cpu::registers::*;
use kmem_region::region::STACK_TOP;
use pie_boot::BootInfo;

use super::debug;
use crate::{
    ArchIf,
    arch::{
        Arch, cache,
        paging::{set_kernel_table, set_user_table},
    },
    entry,
    mem::{page::new_mapped_table, setup_memory_regions, stack_top_cpu0},
    platform::*,
    printkv, println,
};

pub fn primary_entry(boot_info: BootInfo) {
    unsafe {
        cache::dcache_all(cache::DcacheOp::CleanAndInvalidate);
        println!();
        printkv!("Current EL", "{}", CurrentEL.read(CurrentEL::EL));

        entry::setup(boot_info);

        let sp = stack_top_cpu0();

        printkv!("Stack top", "{:?}", sp);

        // SP 移动到物理地址正确位置
        asm!(
            "MOV SP, {sp}",
            "B   {fix_sp}",
            sp = in(reg) sp.raw(),
            fix_sp = sym phys_sp_entry,
            options(noreturn, nostack),
        )
    }
}

fn phys_sp_entry() -> ! {
    println!("SP moved");

    setup_timer();

    setup_memory_regions(Arch::cpu_id(), cpu_list().unwrap());

    println!("Memory regions setup done!");

    let table = new_mapped_table(false);

    println!("Mov sp to {:#x}", STACK_TOP);

    debug::reloacte();

    set_kernel_table(table);
    set_user_table(0usize.into());

    unsafe {
        asm!(
            "MOV SP, {sp}",
            "B {f}",
            sp = const STACK_TOP,
            f = sym crate::to_main,
            options(nostack, noreturn),
        );
    }
}

fn setup_timer() {
    #[cfg(not(feature = "vm"))]
    {
        CNTP_CTL_EL0.write(CNTP_CTL_EL0::ENABLE::SET);
        CNTP_TVAL_EL0.set(0);
    }
    #[cfg(feature = "vm")]
    {
        unsafe {
            // ENABLE, bit [0], Enables the timer.
            // * 0b0: Timer disabled.
            // * 0b1: Timer enabled.
            core::arch::asm!("msr CNTHP_CTL_EL2, {0:x}", in(reg) 0b1);
            core::arch::asm!("msr CNTHP_TVAL_EL2, {0:x}", in(reg) 0);
        }
    }
}
