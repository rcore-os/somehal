#![no_std]

pub use pie_boot::{BootInfo, MemoryKind, MemoryRegion, boot_text, boot_data};
pub use somehal_macros::entry_rt as entry;

#[allow(clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
pub unsafe extern "Rust" fn __vma_relocate_entry(boot_info: BootInfo) {
    unsafe extern "Rust" {
        fn __somert_main(boot_info: BootInfo);
    }

    unsafe {
        __somert_main(boot_info);
    }
}
