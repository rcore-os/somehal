use core::arch::asm;

use pie_boot::BootInfo;
use riscv::register::satp;

use crate::{
    entry,
    mem::{page::new_mapped_table, setup_memory_regions, stack_top_phys, stack_top_virt},
    platform::*,
    printkv, println,
};

pub fn primary_entry(boot_info: BootInfo) {
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

    let sp = stack_top_phys(CpuIdx::primary());

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

    let table = new_mapped_table(true);
    let sp = stack_top_virt(CpuIdx::primary());
    println!("Mov sp to {:?}", sp);

    let mut old = satp::read();
    old.set_ppn(table.raw() >> 12);

    unsafe {
        asm!("mv    t1,  {satp}",
             "la     t2, {entry}",
             "mv    t3,  {sp}",
            satp = in(reg) old.bits(),
            entry = sym crate::entry::entry_virt_and_liner,
            sp = in(reg) sp.raw(),
        );
        asm!(
            "mv    sp, t3",
            "sfence.vma",
            "csrw satp, t1",
            "jalr  t2",
            "j     .",
            options(nostack, nomem, noreturn),
        )
    }
}
