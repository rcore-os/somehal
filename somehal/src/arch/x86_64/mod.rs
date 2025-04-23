use core::arch::asm;

use entry::primary_entry;
use kmem_region::{IntAlign, region::MemConfig};
use log::trace;
use page_table_generic::TableGeneric;
use paging::new_pte_with_config;
use pie_boot::BootInfo;
use x86::controlregs;

use crate::{
    ArchIf,
    mem::{PhysAddr, VirtAddr, page::page_size},
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
        new_pte_with_config(config)
    }

    fn set_kernel_table(addr: PhysAddr) {
        let old_root = Self::get_kernel_table();
        trace!("set page table root: {:?} => {:?}", old_root, addr);
        if old_root != addr {
            unsafe { controlregs::cr3_write(addr.raw() as _) }
        }
    }

    fn get_kernel_table() -> PhysAddr {
        unsafe { controlregs::cr3() as usize }
            .align_down(page_size())
            .into()
    }

    fn set_user_table(_addr: PhysAddr) {
        todo!()
    }

    fn get_user_table() -> PhysAddr {
        todo!()
    }

    fn flush_tlb(vaddr: Option<VirtAddr>) {
        Self::PageTable::flush(vaddr.map(|o| o.raw().into()));
    }

    fn wait_for_event() {
        unsafe { asm!("hlt") }
    }

    fn init_debugcon() {
        uart16550::init();
    }

    fn cpu_id() -> crate::platform::CpuId {
        match raw_cpuid::CpuId::new().get_feature_info() {
            Some(finfo) => (finfo.initial_local_apic_id() as usize).into(),
            None => crate::platform::CpuId::new(0),
        }
    }

    fn primary_entry(boot_info: BootInfo) {
        primary_entry(boot_info);
    }
}
