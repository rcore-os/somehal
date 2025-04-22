pub use crate::mem::{PhysAddr, VirtAddr};
use crate::platform::CpuId;
use kmem_region::region::MemConfig;
pub use page_table_generic::*;
use pie_boot::BootInfo;

pub trait ArchIf {
    fn early_debug_put(byte: u8);

    type PageTable: TableGeneric;

    fn new_pte_with_config(config: MemConfig) -> <Self::PageTable as TableGeneric>::PTE;

    fn set_kernel_table(addr: PhysAddr);

    fn get_kernel_table() -> PhysAddr;

    fn set_user_table(addr: PhysAddr);

    fn get_user_table() -> PhysAddr;

    fn flush_tlb(vaddr: Option<VirtAddr>);

    fn wait_for_event();

    fn init_debugcon();

    fn cpu_id() -> CpuId;

    fn primary_entry(boot_info: BootInfo);
}
