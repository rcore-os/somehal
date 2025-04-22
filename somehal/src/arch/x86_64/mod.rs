use core::arch::{asm, global_asm};

use entry::primary_entry;
use kmem_region::region::MemConfig;
use page_table_generic::TableGeneric;
use pie_boot::BootInfo;

use crate::{
    ArchIf,
    mem::{PhysAddr, VirtAddr},
};

mod entry;
pub(crate) mod paging;
mod uart16550;

pub struct Arch;

impl ArchIf for Arch {
    fn early_debug_put(byte: u8) {
        uart16550::write_bytes(&[byte]);
    }

    type PageTable = paging::Table;

    fn new_pte_with_config(config: MemConfig) -> <Self::PageTable as TableGeneric>::PTE {
        todo!()
    }

    fn set_kernel_table(addr: PhysAddr) {
        todo!()
    }

    fn get_kernel_table() -> PhysAddr {
        todo!()
    }

    fn set_user_table(addr: PhysAddr) {
        todo!()
    }

    fn get_user_table() -> PhysAddr {
        todo!()
    }

    fn flush_tlb(vaddr: Option<VirtAddr>) {
        todo!()
    }

    fn wait_for_event() {
        unsafe { asm!("hlt") }
    }

    fn init_debugcon() {
        todo!()
    }

    fn cpu_id() -> crate::platform::CpuId {
        todo!()
    }

    fn primary_entry(boot_info: BootInfo) {
        primary_entry(boot_info);
    }
}
