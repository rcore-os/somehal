use riscv::{
    asm::{sfence_vma, sfence_vma_all},
    register::satp,
};

use crate::ArchIf;

mod boot;
mod entry;
pub(crate) mod paging;

pub struct Arch;

impl ArchIf for Arch {
    fn early_debug_put(byte: u8) {
        sbi_rt::console_write_byte(byte);
    }

    fn is_mmu_enabled() -> bool {
        paging::is_mmu_enabled()
    }

    type PageTable = paging::Table;

    fn new_pte_with_config(
        config: kmem::region::MemConfig,
    ) -> <Self::PageTable as kmem::paging::TableGeneric>::PTE {
        paging::new_pte_with_config(config)
    }

    fn set_kernel_table(addr: kmem::PhysAddr) {
        todo!()
    }

    fn get_kernel_table() -> kmem::PhysAddr {
        todo!()
    }

    fn set_user_table(addr: kmem::PhysAddr) {
        todo!()
    }

    fn get_user_table() -> kmem::PhysAddr {
        todo!()
    }

    #[inline(always)]
    fn flush_tlb(vaddr: Option<kmem::VirtAddr>) {
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
}
