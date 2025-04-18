use core::{
    arch::{asm, naked_asm},
    ptr::null_mut,
};

use kmem::region::STACK_TOP;
use riscv::register::satp;

use crate::{
    fdt, handle_err,
    mem::{
        boot::set_kcode_va_offset, kernal_load_start_link_addr, page::new_mapped_table,
        setup_memory_main, setup_memory_regions, stack_top_cpu0,
    },
    println,
    vec::ArrayVec,
};

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __vma_relocate_entry(hartid: usize, kcode_offset: usize, dtb: *mut u8) {
    unsafe {
        println!("MMU ready!");

        asm!(
            "add  gp, gp, {offset}",
            offset = in(reg) kcode_offset,
            options(nostack)
        );
        set_kcode_va_offset(kcode_offset);
        fdt::set_fdt_ptr(dtb);
    }

    println!(
        "{:<12}: {:#X}",
        "Kernel LMA",
        kernal_load_start_link_addr() - kcode_offset
    );

    println!("{:<12}: {:#X}", "Code offst", kcode_offset);
    println!("{:<12}: {:?}", "Hart", hartid);

    println!("{:<12}: {:?}", "FDT", dtb);

    let cpu_count = handle_err!(fdt::cpu_count(), "could not get cpu count");

    println!("{:<12}: {}", "CPU count", cpu_count);

    let memories = handle_err!(fdt::find_memory(), "could not get memories");

    setup_memory_main(memories, cpu_count);

    let sp = stack_top_cpu0();

    println!("{:<12}: {:?}", "Stack top", sp);

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
    let mut rsv = ArrayVec::<_, 4>::new();

    if let Some(r) = fdt::save_fdt() {
        let _ = rsv.try_push(r);
    }

    setup_memory_regions(hartid.into(), rsv, fdt::cpu_list().unwrap());

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
