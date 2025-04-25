use core::arch::asm;

use kmem_region::region::STACK_TOP;
use pie_boot::BootInfo;
use riscv::register::satp;

use crate::{
    arch::debug_init,
    entry,
    mem::{page::new_mapped_table, setup_memory_regions, stack_top_cpu0},
    platform::*,
    printkv, println,
};

pub fn primary_entry(boot_info: BootInfo) {
    debug_init();
    unsafe {
        println!();

        asm!(
            "add  gp, gp, {offset}",
            offset = in(reg) boot_info.kcode_offset,
            options(nostack)
        );
    }
    let hartid = boot_info.cpu_id;

    entry::setup(boot_info);

    let sp = stack_top_cpu0();

    printkv!("Stack top", "{:?}", sp);

    // SP 移动到物理地址正确位置
    unsafe {
        asm!("mv  t1, {}", in(reg) hartid);
        asm!(
            "mv  sp, {sp}",
            "mv  a0, t1",
            "call   {fix_sp}",
            sp = in(reg) sp.raw(),
            fix_sp = sym phys_sp_entry,
            options(noreturn, nostack),
        )
    }
}

fn phys_sp_entry(hartid: usize) -> ! {
    println!("SP moved");

    setup_memory_regions(hartid.into(), cpu_list().unwrap());

    println!("Memory regions setup done!");

    let table = new_mapped_table(false);

    println!("Mov sp to {:#x}", STACK_TOP);
    let mut old = satp::read();
    old.set_ppn(table.raw() >> 12);

    unsafe {
        asm!("mv    t1,  {satp}",
             "la     t2, {entry}",
            satp = in(reg) old.bits(),
            entry = sym crate::__somehal_main
        );
        asm!(
            "li    sp, {sp}",
            "sfence.vma",
            "csrw satp, t1",
            "jalr  t2",
            "j     .",
            sp = const STACK_TOP,
            options(nostack, nomem, noreturn),
        )
    }
}
