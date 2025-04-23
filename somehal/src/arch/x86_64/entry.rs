use core::arch::asm;

use kmem_region::region::{STACK_TOP, set_kcode_va_offset};
use pie_boot::{BootInfo, MemoryKind};

use crate::{
    ArchIf,
    arch::Arch,
    mem::{
        PhysMemory, clean_bss, page::new_mapped_table, setup_memory_main, setup_memory_regions,
        stack_top_cpu0,
    },
    platform::{self, cpu_list},
    printkv, println,
};

pub fn primary_entry(boot_info: BootInfo) -> ! {
    unsafe {
        clean_bss();
        set_kcode_va_offset(boot_info.kcode_offset);
        Arch::init_debugcon();

        println!("\r\nMMU ready");

        platform::init();

        let cpu_count = platform::cpu_count();

        let reserved_memories = boot_info.memory_regions.clone().filter_map(|o| {
            if matches!(o.kind, MemoryKind::Reserved) {
                Some(PhysMemory {
                    addr: o.start.into(),
                    size: o.end - o.start,
                })
            } else {
                None
            }
        });

        let memories = boot_info.memory_regions.filter_map(|o| {
            if matches!(o.kind, MemoryKind::Avilable) {
                Some(PhysMemory {
                    addr: o.start.into(),
                    size: o.end - o.start,
                })
            } else {
                None
            }
        });

        setup_memory_main(reserved_memories, memories, cpu_count);

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

    let cpu_id = Arch::cpu_id();

    printkv!("CPU ID", "{:?}", cpu_id);

    setup_memory_regions(cpu_id, cpu_list());

    let table = new_mapped_table();

    println!("Mov sp to {:#x}", STACK_TOP);

    Arch::set_kernel_table(table);
    Arch::flush_tlb(None);

    unsafe {
        asm!(
            "mov rsp, {sp}",
            "jmp {f}",
            sp = const STACK_TOP,
            f = sym crate::__somehal_main,
            options(nostack, noreturn),
        );
    }
}
