use core::arch::asm;

use kmem_region::region::set_kcode_va_offset;
use pie_boot::{BootInfo, MemoryKind};

use crate::{
    ArchIf,
    arch::{Arch, uart16550},
    mem::{PhysMemory, setup_memory_main, setup_memory_regions, stack_top_cpu0},
    platform::{self, cpu_list},
    printkv, println,
};

pub fn primary_entry(boot_info: BootInfo) -> ! {
    unsafe {
        set_kcode_va_offset(boot_info.kcode_offset);
        uart16550::init();

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

    unreachable!()
}
