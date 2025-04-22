use pie_boot::BootInfo;

use crate::{ArchIf, arch::Arch};

#[unsafe(no_mangle)]
pub unsafe extern "Rust" fn __vma_relocate_entry(boot_info: BootInfo) {
    Arch::primary_entry(boot_info);
}
