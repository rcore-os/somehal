pub use crate::mem::{PhysAddr, VirtAddr};
pub use kmem::paging::*;
use kmem::region::MemConfig;

pub trait ArchIf {
    fn early_debug_put(byte: u8);
    fn is_mmu_enabled() -> bool;
    type PageTable: TableGeneric;
    fn new_pte_with_config(config: MemConfig) -> <Self::PageTable as TableGeneric>::PTE;
    fn set_kernel_table(addr: PhysAddr);

    fn get_kernel_table() -> PhysAddr;

    fn set_user_table(addr: PhysAddr);

    fn get_user_table() -> PhysAddr;

    fn flush_tlb(vaddr: Option<VirtAddr>);

    fn wait_for_event();

    fn init_debugcon();
}
