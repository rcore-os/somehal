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
        debug::init();
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
            f = sym crate::__somehal_main,
            options(nostack, noreturn),
        );
    }
}
