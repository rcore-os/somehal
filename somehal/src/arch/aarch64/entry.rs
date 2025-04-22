use core::arch::asm;

use aarch64_cpu::registers::*;
use heapless::Vec;
use kmem::region::{
    AccessFlags, CacheConfig, MemConfig, MemRegionKind, STACK_TOP, kcode_offset,
    set_kcode_va_offset,
};
use pie_boot::BootInfo;

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
        kernal_load_start_link_addr, main_memory::RegionAllocator, page::new_mapped_table,
        setup_memory_main, setup_memory_regions, stack_top_cpu0,
    },
    println,
};

#[unsafe(no_mangle)]
// pub unsafe extern "C" fn __vma_relocate_entry(kcode_offset: usize, dtb: *mut u8) {
pub unsafe extern "C" fn __vma_relocate_entry(boot_info: *const BootInfo) {
    unsafe {
        let boot_info = &*boot_info;

        set_kcode_va_offset(boot_info.kcode_offset);
        fdt::set_fdt_ptr(boot_info.fdt.unwrap().as_ptr());
        debug::init();

        println!("MMU ready!");

        println!(
            "{:<12}: {:#X}",
            "Kernel LMA",
            kernal_load_start_link_addr() - boot_info.kcode_offset
        );

        println!("{:<12}: {}", "Current EL", CurrentEL.read(CurrentEL::EL));

        let cpu_count = handle_err!(fdt::cpu_count(), "could not get cpu count");

        println!("{:<12}: {}", "CPU count", cpu_count);

        let memories = handle_err!(fdt::find_memory(), "could not get memories");

        setup_memory_main(boot_info.main_memory_free_start, memories, cpu_count);

        let sp = stack_top_cpu0();

        println!("{:<12}: {:?}", "Stack top", sp);

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
    let mut rsv = Vec::<_, 4>::new();

    let alloc = RegionAllocator::new(
        "rsv",
        MemConfig {
            access: AccessFlags::Read,
            cache: CacheConfig::Normal,
        },
        MemRegionKind::Code,
        kcode_offset(),
    );

    if let Some(r) = save_fdt(&alloc) {
        // let _ = rsv.push(r);
    }

    let _ = rsv.push(alloc.into());
    let _ = rsv.push(super::debug::MEM_REGION_DEBUG_CON.clone());

    setup_memory_regions(Arch::cpu_id(), rsv.into_iter(), fdt::cpu_list().unwrap());

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
