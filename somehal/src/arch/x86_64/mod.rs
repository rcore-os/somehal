use crate::ArchIf;

mod entry;
mod multiboot1;
pub(crate) mod paging;

extern crate pie_boot;

pub struct Arch;

impl ArchIf for Arch {
    fn early_debug_put(byte: u8) {
        todo!()
    }

    type PageTable = paging::Table;

    fn new_pte_with_config(
        config: kmem::region::MemConfig,
    ) -> <Self::PageTable as kmem::paging::TableGeneric>::PTE {
        todo!()
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

    fn flush_tlb(vaddr: Option<kmem::VirtAddr>) {
        todo!()
    }

    fn wait_for_event() {
        todo!()
    }

    fn init_debugcon() {
        todo!()
    }

    fn cpu_id() -> crate::platform::CpuId {
        todo!()
    }
}
