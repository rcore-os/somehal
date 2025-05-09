use kmem_region::region::*;
use pie_boot::{BootInfo, MemoryKind};

use crate::{
    ArchIf, CpuId, CpuIdx, CpuOnArg,
    arch::Arch,
    handle_err,
    mem::{page::new_mapped_table, *},
    platform, printkv,
};

static mut BOOT_CPU: CpuId = CpuId::new(0);

#[unsafe(no_mangle)]
pub unsafe extern "Rust" fn __vma_relocate_entry(boot_info: BootInfo) {
    unsafe {
        clean_bss();
        set_kcode_va_offset(boot_info.kcode_offset);
        BOOT_CPU = boot_info.cpu_id.into();
        platform::init(&boot_info);
    }
    Arch::init_debugcon();
    Arch::primary_entry(boot_info);
}

pub fn setup(boot_info: BootInfo) {
    printkv!(
        "Kernel LMA",
        "{:#X}",
        kernal_load_start_link_addr() - boot_info.kcode_offset
    );

    printkv!("Code offst", "{:#X}", kcode_offset());

    printkv!("Hart", "{:#x}", boot_info.cpu_id);

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

pub fn entry_virt_and_liner() {
    // 移除低地址空间线性映射
    let table = new_mapped_table(false);

    crate::mem::percpu::setup_stack_and_table();

    Arch::set_kernel_table(table);
    Arch::set_user_table(0usize.into());
    let cpu_idx = CpuIdx::primary();

    let arg = CpuOnArg {
        cpu_id: unsafe { BOOT_CPU },
        cpu_idx,
        page_table: 0usize.into(),
        page_table_with_liner: 0usize.into(),
        stack_top_virt: stack_top_virt(cpu_idx),
    };
    crate::to_main(&arg)
}
