use core::arch::asm;

use kmem_region::region::{STACK_TOP, set_kcode_va_offset};
use pie_boot::{BootInfo, MemoryKind};
use riscv::register::satp;

use crate::{
    arch::debug_init,
    handle_err,
    mem::{
        PhysMemory, kernal_load_start_link_addr, page::new_mapped_table, setup_memory_main,
        setup_memory_regions, stack_top_cpu0,
    },
    platform::*,
    printkv, println,
};

pub fn primary_entry(boot_info: BootInfo) {
    let hartid = boot_info.cpu_id;
    let kcode_offset = boot_info.kcode_offset;

    debug_init();
    unsafe {
        println!("MMU ready!");

        asm!(
            "add  gp, gp, {offset}",
            offset = in(reg) kcode_offset,
            options(nostack)
        );
        set_kcode_va_offset(kcode_offset);
        set_fdt_info(boot_info.fdt);
    }

    printkv!(
        "Kernel LMA",
        "{:#X}",
        kernal_load_start_link_addr() - kcode_offset
    );

    printkv!("Code offst", "{:#X}", kcode_offset);
    printkv!("Hart", "{:?}", hartid);

    if let Some(fdt) = boot_info.fdt {
        printkv!("FDT", "{:?}", fdt.0);
    }

    let cpu_count = handle_err!(cpu_count(), "could not get cpu count");

    printkv!("CPU count", "{}", cpu_count);
    printkv!("Memory start", "{:#x}", boot_info.highest_address);

    let memories = handle_err!(find_memory(), "could not get memories");

    let reserved_memories = boot_info.memory_regions.filter_map(|o| {
        if matches!(o.kind, MemoryKind::Reserved) {
            Some(PhysMemory {
                addr: o.start.into(),
                size: o.end - o.start,
            })
        } else {
            None
        }
    });

    setup_memory_main(reserved_memories, memories.into_iter(), cpu_count);

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

    let table = new_mapped_table();

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
