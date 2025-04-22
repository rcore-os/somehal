use kmem_region::region::set_kcode_va_offset;
use pie_boot::BootInfo;

use crate::{
    ArchIf,
    arch::{Arch, uart16550},
    mem::{init_heap, set_memory_main},
    platform, printkv, println,
};

pub fn primary_entry(boot_info: BootInfo) -> ! {
    unsafe {
        set_kcode_va_offset(boot_info.kcode_offset);
        uart16550::init();

        println!("\r\nMMU ready");

        if let Some(main_end) = boot_info.main_memory_free_end {
            printkv!(
                "main memory",
                "[{:?}, {:?})",
                boot_info.main_memory_free_start,
                main_end
            );
            set_memory_main(boot_info.main_memory_free_start, main_end);
            init_heap();
        }

        platform::init();

        let cpu_list = platform::cpu_list();

        for cpu in cpu_list {
            println!("cpu {:?}", cpu);
        }

        Arch::wait_for_event();
        unreachable!()
    }
}
