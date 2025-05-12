use entry::primary_entry;
use page_table_generic::TableGeneric;
use riscv::{
    asm::{sfence_vma, sfence_vma_all},
    register::{satp, time},
};

use crate::{
    ArchIf,
    mem::{self, page::is_relocated, virt_to_phys},
    platform::CpuId,
};

mod entry;

pub(crate) mod paging;

static mut EXT_CONSOLE: bool = false;

pub struct Arch;

impl ArchIf for Arch {
    #[allow(deprecated)]
    fn early_debug_put(bytes: &[u8]) {
        unsafe {
            if EXT_CONSOLE {
                if is_relocated() {
                    let mut ptr = virt_to_phys(bytes.as_ptr().into());
                    let end = ptr + bytes.len();
                    while ptr < end {
                        let ret =
                            sbi_rt::console_write(sbi_rt::Physical::new(end - ptr, ptr.raw(), 0));
                        if ret.is_err() {
                            return;
                        }
                        ptr += ret.value;
                    }
                } else {
                    for &byte in bytes {
                        sbi_rt::console_write_byte(byte as _);
                    }
                }
            } else {
                for &b in bytes {
                    sbi_rt::legacy::console_putchar(b as _);
                }
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

    fn init_debugcon() {
        let info = sbi_rt::probe_extension(sbi_rt::Console);
        unsafe {
            EXT_CONSOLE = info.is_available();
        }
    }

    fn cpu_id() -> CpuId {
        mem::cpu_id()
    }

    fn primary_entry(boot_info: pie_boot::BootInfo) {
        primary_entry(boot_info);
    }
    
    fn current_ticks() -> u64 {
        time::read() as u64
    }
    
    fn tick_hz() -> u64 {
        10_000_000
    }
    
    fn start_secondary_cpu(
        cpu: CpuId,
        stack: kmem_region::PhysAddr,
    ) -> Result<(), alloc::boxed::Box<dyn core::error::Error>> {
        todo!()
    }
}
