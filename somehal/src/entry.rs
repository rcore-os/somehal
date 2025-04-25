use kmem_region::region::*;
use pie_boot::{BootInfo, MemoryKind};

use crate::{ArchIf, arch::Arch, handle_err, mem::*, platform, printkv};

#[unsafe(no_mangle)]
pub unsafe extern "Rust" fn __vma_relocate_entry(boot_info: BootInfo) {
    unsafe {
        clean_bss();
        set_kcode_va_offset(boot_info.kcode_offset);
        platform::init(&boot_info);
    }

    Arch::primary_entry(boot_info);
}

pub fn setup(boot_info: BootInfo) {
    printkv!(
        "Kernel LMA",
        "{:#X}",
        kernal_load_start_link_addr() - boot_info.kcode_offset
    );

    printkv!("Code offst", "{:#X}", kcode_offset());

    printkv!("Hart", "{:?}", boot_info.cpu_id);

    let cpu_count = handle_err!(platform::cpu_count());

    printkv!("CPU Count", "{}", cpu_count);

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

    let memories1 = boot_info.memory_regions.filter_map(|o| {
        if matches!(o.kind, MemoryKind::Avilable) {
            Some(PhysMemory {
                addr: o.start.into(),
                size: o.end - o.start,
            })
        } else {
            None
        }
    });

    let memories2 = handle_err!(platform::find_memory(), "could not get memories");

    let memories = memories1.chain(memories2);

    setup_memory_main(reserved_memories, memories, cpu_count);
}
