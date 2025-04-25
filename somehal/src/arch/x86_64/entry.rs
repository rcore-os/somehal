use core::arch::asm;

use kmem_region::region::{STACK_TOP, kcode_offset, set_kcode_va_offset};
use pie_boot::{BootInfo, MemoryKind};

use crate::{
    arch::{idt::init_idt, Arch}, mem::{
        clean_bss, kernal_load_start_link_addr, page::{new_mapped_table, set_is_relocated}, setup_memory_main, setup_memory_regions, stack_top_cpu0, PhysMemory
    }, platform::{self, cpu_list}, printkv, println, ArchIf
};

pub fn primary_entry(boot_info: BootInfo) -> ! {
    unsafe {
        clean_bss();
        set_kcode_va_offset(boot_info.kcode_offset);
        platform::init();

        println!("\r\nMMU ready");

        printkv!(
            "Kernel LMA",
            "{:#X}",
            kernal_load_start_link_addr() - boot_info.kcode_offset
        );

        printkv!("Code offst", "{:#X}", kcode_offset());

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
    setup();
    let table = new_mapped_table(true);
    Arch::set_kernel_table(table);
    unsafe {
        x86::tlb::flush_all();
    }
    let sp = STACK_TOP;
    printkv!("Stack top", "{:#x}", sp);

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
    set_is_relocated();
    let table = new_mapped_table(false);
    Arch::set_kernel_table(table);
    unsafe {
        x86::tlb::flush_all();
    }

    unsafe { crate::__somehal_main() }
}
