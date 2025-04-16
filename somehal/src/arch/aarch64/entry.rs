use core::arch::asm;

use aarch64_cpu::registers::*;
use kmem::region::STACK_TOP;

use super::debug;
use crate::{
    ArchIf,
    arch::{
        Arch,
        paging::{set_kernel_table, set_user_table},
    },
    fdt::{self, save_fdt},
    handle_err,
    mem::{
        boot::set_kcode_va_offset, kernal_load_start_link_addr, page::new_mapped_table,
        setup_memory_main, setup_memory_regions, stack_top_cpu0,
    },
    println,
    vec::ArrayVec,
};

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __vma_relocate_entry(kcode_offset: usize, dtb: *mut u8) {
    unsafe {
        set_kcode_va_offset(kcode_offset);
        fdt::set_fdt_ptr(dtb);
    }
    debug::init();

    println!("MMU ready!");

    println!(
        "{:<12}: {:#X}",
        "Kernel LMA",
        kernal_load_start_link_addr() - kcode_offset
    );

    println!("{:<12}: {}", "Current EL", CurrentEL.read(CurrentEL::EL));

    let cpu_count = handle_err!(fdt::cpu_count(), "could not get cpu count");

    println!("{:<12}: {}", "CPU count", cpu_count);

    let memories = handle_err!(fdt::find_memory(), "could not get memories");

    setup_memory_main(memories, cpu_count);

    let sp = stack_top_cpu0();

    println!("{:<12}: {:?}", "Stack top", sp);

    // SP 移动到物理地址正确位置
    unsafe {
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
    let mut rsv = ArrayVec::<_, 4>::new();

    if let Some(r) = save_fdt() {
        let _ = rsv.try_push(r);
    }

    let _ = rsv.try_push(super::debug::MEM_REGION_DEBUG_CON.clone());

    setup_memory_regions(Arch::cpu_id(), rsv, fdt::cpu_list().unwrap());

    println!("Memory regions setup done!");

    let table = new_mapped_table();

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
