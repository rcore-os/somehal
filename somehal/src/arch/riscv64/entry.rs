use core::{arch::asm, ptr::null_mut};

use heapless::Vec;
use kmem_region::region::{
    AccessFlags, CacheConfig, MemConfig, MemRegionKind, STACK_TOP, kcode_offset,
    set_kcode_va_offset,
};
use pie_boot::BootInfo;
use riscv::register::satp;

use crate::{
    arch::debug_init,
    handle_err,
    mem::{
        kernal_load_start_link_addr, main_memory::RegionAllocator, page::new_mapped_table,
        setup_memory_main, setup_memory_regions, stack_top_cpu0,
    },
    platform::*,
    println,
};

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __vma_relocate_entry(boot_info: *const BootInfo) {
    let boot_info = unsafe { &*boot_info };
    let hartid = boot_info.cpu_id;
    let kcode_offset = boot_info.kcode_offset;
    let dtb = boot_info.fdt.map(|t| t.as_ptr()).unwrap_or(null_mut());

    debug_init();
    unsafe {
        println!("MMU ready!");

        asm!(
            "add  gp, gp, {offset}",
            offset = in(reg) kcode_offset,
            options(nostack)
        );
        set_kcode_va_offset(kcode_offset);
        set_fdt_ptr(dtb);
    }

    println!(
        "{:<12}: {:#X}",
        "Kernel LMA",
        kernal_load_start_link_addr() - kcode_offset
    );

    println!("{:<12}: {:#X}", "Code offst", kcode_offset);
    println!("{:<12}: {:?}", "Hart", hartid);

    println!("{:<12}: {:?}", "FDT", dtb);

    let cpu_count = handle_err!(cpu_count(), "could not get cpu count");

    println!("{:<12}: {}", "CPU count", cpu_count);
    println!(
        "{:<12}: {:?}",
        "Memory start", boot_info.main_memory_free_start
    );

    let memories = handle_err!(find_memory(), "could not get memories");

    setup_memory_main(
        boot_info.main_memory_free_start,
        memories.into_iter(),
        cpu_count,
    );

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

    let mut rsv = Vec::<_, 4>::new();

    let mut alloc = RegionAllocator::new(
        "rsv",
        MemConfig {
            access: AccessFlags::Read,
            cache: CacheConfig::Normal,
        },
        MemRegionKind::Code,
        kcode_offset(),
    );
    if save_fdt(&mut alloc).is_none() {
        println!("FDT save failed!");
        panic!();
    }

    let _ = rsv.push(alloc.into());

    setup_memory_regions(hartid.into(), rsv.into_iter(), cpu_list().unwrap());

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
