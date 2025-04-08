use core::arch::asm;

use kmem::region::STACK_TOP;
use riscv::register::satp;

use crate::{
    arch::paging::set_page_table,
    fdt, handle_err,
    mem::{
        boot::{kcode_offset, kernal_load_addr},
        page::new_mapped_table,
        setup_memory_main, setup_memory_regions, stack_top_cpu0,
    },
    println,
    vec::ArrayVec,
};

pub fn mmu_entry() -> ! {
    println!("MMU ready!");
    let offset = kcode_offset();
    unsafe {
        asm!(
            "add  gp, gp, {offset}",
            offset = in(reg) offset,
            options(nostack)
        );
    }

    println!("{:<12}: {:?}", "Kernel LMA", kernal_load_addr());

    let cpu_count = handle_err!(fdt::cpu_count(), "could not get cpu count");

    println!("{:<12}: {}", "CPU count", cpu_count);

    let memories = handle_err!(fdt::find_memory(), "could not get memories");

    setup_memory_main(memories, cpu_count);

    let sp = stack_top_cpu0();

    println!("{:<12}: {:?}", "Stack top", sp);

    // SP 移动到物理地址正确位置
    unsafe {
        asm!(
            "mv  sp, {sp}",
            "call   {fix_sp}",
            sp = in(reg) sp.raw(),
            fix_sp = sym phys_sp_entry,
            options(noreturn, nostack),
        )
    }
}

fn phys_sp_entry() -> ! {
    println!("SP moved");
    let mut rsv = ArrayVec::<_, 4>::new();

    if let Some(r) = fdt::save_fdt() {
        let _ = rsv.try_push(r);
    }

    setup_memory_regions(rsv);

    println!("Memory regions setup done!");

    let table = new_mapped_table();

    println!("Mov sp to {:#x}", STACK_TOP);
    let mut old = satp::read();
    old.set_ppn(table.raw() >> 12);

    unsafe {
        asm!("mv    t1,  {satp}",
             "la     t2,  rust_main ",
            satp = in(reg) old.bits(),
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
