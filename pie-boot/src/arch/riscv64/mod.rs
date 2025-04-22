use crate::archif::ArchIf;

#[macro_use]
mod macros;

mod boot;
mod context;
mod mmu;
mod trap;

use crate::paging::TableGeneric;
pub use boot::*;
use mmu::new_pte_with_config;

static mut EXT_CONSOLE: bool = false;

fn debug_init() {
    let info = sbi_rt::probe_extension(sbi_rt::Console);
    unsafe {
        EXT_CONSOLE = info.is_available();
    }
}

pub struct Arch;

impl ArchIf for Arch {
    #[inline(always)]
    #[allow(deprecated)]
    fn early_debug_put(byte: u8) {
        unsafe {
            if EXT_CONSOLE {
                sbi_rt::console_write_byte(byte);
            } else {
                sbi_rt::legacy::console_putchar(byte as _);
            }
        }
    }

    fn wait_for_event() {
        riscv::asm::wfi();
    }

    type PageTable = mmu::Table;

    fn new_pte_with_config(
        config: kmem_region::region::MemConfig,
    ) -> <Self::PageTable as TableGeneric>::PTE {
        new_pte_with_config(config)
    }
}
