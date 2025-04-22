use page_table_generic::TableGeneric;
use riscv::{
    asm::{sfence_vma, sfence_vma_all},
    register::satp,
};

use crate::{ArchIf, mem, platform::CpuId};

mod boot;
mod entry;

pub(crate) mod paging;

static mut EXT_CONSOLE: bool = false;

fn debug_init() {
    let info = sbi_rt::probe_extension(sbi_rt::Console);
    unsafe {
        EXT_CONSOLE = info.is_available();
    }
}

pub struct Arch;

impl ArchIf for Arch {
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

    type PageTable = paging::Table;

    #[inline(always)]
    fn new_pte_with_config(
        config: kmem_region::region::MemConfig,
    ) -> <Self::PageTable as TableGeneric>::PTE {
        paging::new_pte_with_config(config)
    }

    fn set_kernel_table(_addr: kmem_region::PhysAddr) {
        todo!()
    }

    fn get_kernel_table() -> kmem_region::PhysAddr {
        (satp::read().ppn() << 12).into()
    }

    fn set_user_table(_addr: kmem_region::PhysAddr) {
        todo!()
    }

    fn get_user_table() -> kmem_region::PhysAddr {
        todo!()
    }

    #[inline(always)]
    fn flush_tlb(vaddr: Option<kmem_region::VirtAddr>) {
        if let Some(vaddr) = vaddr {
            sfence_vma(0, vaddr.raw())
        } else {
            sfence_vma_all();
        }
    }

    fn wait_for_event() {
        riscv::asm::wfi();
    }

    fn init_debugcon() {}

    fn cpu_id() -> CpuId {
        mem::cpu_id()
    }
}
