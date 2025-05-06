use aarch64_cpu::asm::wfe;
use entry::primary_entry;
use page_table_generic::TableGeneric;
use pie_boot::BootInfo;

use crate::{
    ArchIf,
    mem::{PhysAddr, VirtAddr},
    platform::CpuId,
};

mod cache;
mod context;
pub mod debug;
mod entry;
mod mp;
mod paging;
mod psci;
mod trap;

pub struct Arch;

use aarch64_cpu::registers::*;

impl ArchIf for Arch {
    fn early_debug_put(b: &[u8]) {
        for &b in b {
            debug::write_byte(b);
        }
    }

    type PageTable = paging::Table;

    fn new_pte_with_config(
        config: kmem_region::region::MemConfig,
    ) -> <Self::PageTable as TableGeneric>::PTE {
        paging::new_pte_with_config(config)
    }

    fn set_kernel_table(addr: PhysAddr) {
        paging::set_kernel_table(addr);
    }

    fn get_kernel_table() -> PhysAddr {
        paging::get_kernel_table()
    }

    fn set_user_table(addr: PhysAddr) {
        paging::set_user_table(addr);
    }

    fn get_user_table() -> PhysAddr {
        paging::get_user_table()
    }

    fn flush_tlb(vaddr: Option<VirtAddr>) {
        paging::flush_tlb(vaddr);
    }

    fn wait_for_event() {
        wfe();
    }

    fn init_debugcon() {
        debug::init();
    }

    fn cpu_id() -> CpuId {
        ((MPIDR_EL1.get() & 0xffffff) as usize).into()
    }

    fn primary_entry(boot_info: BootInfo) {
        primary_entry(boot_info);
    }

    fn current_ticks() -> u64 {
        CNTPCT_EL0.get()
    }

    fn tick_hz() -> u64 {
        CNTFRQ_EL0.get()
    }

    fn start_secondary_cpu(
        cpu: CpuId,
        entry: usize,
        stack_top: usize,
    ) -> Result<(), alloc::boxed::Box<dyn core::error::Error>> {
        mp::cpu_on(cpu, entry, stack_top)
    }
}
