use pie_boot_if::BootInfo;

use crate::{common, lazy_static::LazyStatic, power, println, setup_exception_vectors};

static BOOT_INFO: LazyStatic<BootInfo> = LazyStatic::new();

pub fn boot_info() -> &'static BootInfo {
    &BOOT_INFO
}

pub fn virt_entry(args: &BootInfo) {
    common::mem::clean_bss();
    BOOT_INFO.init(args.clone());
    common::fdt::init_debugcon(boot_info().fdt);
    println!("SomeHAL booting...");
    setup_exception_vectors();
    power::init_by_fdt(boot_info().fdt);
    common::fdt::setup_plat_info();
    common::mem::init_regions(&args.memory_regions);

    unsafe {
        let (region_ptr, region_len) =
            common::mem::with_regions(|regions| (regions.as_mut_ptr(), regions.len()));
        let region_slice = core::slice::from_raw_parts_mut(region_ptr, region_len);
        BOOT_INFO.edit(|info| info.memory_regions = region_slice.into());

        crate::BOOT_PT = BOOT_INFO.pg_start as usize;

        unsafe extern "Rust" {
            fn __pie_boot_main(args: &BootInfo);
        }
        println!("Goto main...");
        __pie_boot_main(&BOOT_INFO);

        power::shutdown();
    }
}
