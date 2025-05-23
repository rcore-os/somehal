use core::arch::asm;

use entry::primary_entry;
use kmem_region::IntAlign;
use paging::new_pte_with_config;

use crate::{archif::*, mem::page::page_size, platform};

mod context;
mod debug;
mod entry;
mod idt;
pub(crate) mod paging;
// mod trap;

pub struct Arch;

impl ArchIf for Arch {
    fn early_debug_put(bytes: &[u8]) {
        for &b in bytes {
            debug::write_byte(b);
        }
    }

    type PageTable = paging::Table;

    fn new_pte_with_config(config: MemConfig) -> <Self::PageTable as TableGeneric>::PTE {
        new_pte_with_config(config)
    }

    #[inline(always)]
    fn set_kernel_table(addr: PhysAddr) {
        let old_root = Self::get_kernel_table();
        if old_root != addr {
            unsafe {
                asm!("mov {0}, %cr3", in(reg) addr.raw(), options(att_syntax));
            }
        }
    }

    #[inline(always)]
    fn get_kernel_table() -> PhysAddr {
        unsafe {
            let ret: usize;
            asm!("mov %cr3, {0}", out(reg) ret, options(att_syntax));
            ret
        }
        .align_down(page_size())
        .into()
    }

    fn set_user_table(_addr: PhysAddr) {
        todo!()
    }

    fn get_user_table() -> PhysAddr {
        todo!()
    }

    #[inline(always)]
    fn flush_tlb(vaddr: Option<VirtAddr>) {
        unsafe {
            if let Some(vaddr) = vaddr {
                x86::tlb::flush(vaddr.raw());
            } else {
                x86::tlb::flush_all();
            }
        }
    }

    fn wait_for_event() {
        unsafe { asm!("hlt") }
    }

    fn init_debugcon() {
        debug::init();
        platform::init_debugcon();
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

    fn current_ticks() -> u64 {
        unsafe { core::arch::x86_64::_rdtsc() } 
    }

    fn tick_hz() -> u64 {
        4_000_000_000
    }

    fn start_secondary_cpu(
        cpu: CpuId,
        entry: usize,
        stack: usize,
    ) -> Result<(), alloc::boxed::Box<dyn core::error::Error>> {
        todo!()
    }
}
