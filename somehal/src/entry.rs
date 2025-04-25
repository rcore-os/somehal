use boot_api::BootInfo;
use kmem_region::MB;

use crate::{ArchIf, arch::Arch};

const LOADER_SIZE: usize = 2 * MB - 1;

#[unsafe(link_section = ".loader")]
#[used(linker)]
static LOADER: [u8; LOADER_SIZE] = new_loader();

const fn new_loader() -> [u8; LOADER_SIZE] {
    let mut loader = [0; LOADER_SIZE];

    let src = include_bytes!(concat!(env!("OUT_DIR"), "/loader.bin"));

    let len = src.len();
    let mut i = 0;
    while i < len {
        loader[i] = src[i];
        i += 1;
    }

    loader
}

#[unsafe(no_mangle)]
pub unsafe extern "Rust" fn __vma_relocate_entry(boot_info: BootInfo) {
    Arch::primary_entry(boot_info);
}
