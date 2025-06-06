use pie_boot::BootInfo;

#[unsafe(no_mangle)]
pub unsafe extern "Rust" fn __vma_relocate_entry(boot_info: BootInfo) {
    loop {}
}
