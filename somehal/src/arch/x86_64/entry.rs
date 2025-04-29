use core::arch::asm;

use kmem_region::region::STACK_TOP;
use pie_boot::BootInfo;

use crate::{
    ArchIf,
    arch::{Arch, idt::init_idt},
    entry,
    mem::{page::new_mapped_table, setup_memory_regions, stack_top_cpu0},
    platform::cpu_list,
    printkv, println,
};

pub fn primary_entry(boot_info: BootInfo) -> ! {
    unsafe {
        println!();
       
        entry::setup(boot_info);

        let sp = stack_top_cpu0();

        printkv!("Stack top", "{:?}", sp);

        asm!(
            "mov rsp, rdi",
            "jmp {entry}",
            entry = sym phys_sp_entry,
            in("rdi") sp.raw(),
            options(noreturn)
        )
    }
}

fn phys_sp_entry() -> ! {
    println!("SP moved");
    setup();
    let sp = STACK_TOP;
    printkv!("Stack top", "{:#x}", sp);

    let table = new_mapped_table(true);
    Arch::set_kernel_table(table);

    unsafe {
        asm!(
            "mov rsp, rdi",
            "jmp {entry}",
            entry = sym virt_sp_entry,
            in("rdi") sp,
            options(noreturn)
        )
    }
}

fn setup() {
    init_idt();
    let cpu_id = Arch::cpu_id();
    printkv!("CPU ID", "{:?}", cpu_id);
    setup_memory_regions(cpu_id, cpu_list());
}

fn virt_sp_entry() -> ! {
    // 移除低地址空间线性映射
    let table = new_mapped_table(false);
    Arch::set_kernel_table(table);
    unsafe {
        x86::tlb::flush_all();
    }

    crate::to_main()
}
