use core::arch::asm;

use aarch64_cpu::{asm::barrier::*, registers::*};

use super::entry::mmu_entry;

cfg_match! {
    feature = "vm" =>{

    }
    _ =>{
        mod table_el1;
        pub use table_el1::*;
    }
}

#[link_boot::link_boot]
mod _m {

    use crate::{dbgln, fdt::fdt_size, mem::boot::new_boot_table};

    /// 参数为目标虚拟地址
    #[inline(always)]
    pub fn enable_mmu() -> ! {
        setup_regs();
        let table = new_boot_table(fdt_size());

        dbgln!("Set kernel table {}", table.raw());
        set_kernel_table(table);
        set_user_table(table);
        flush_tlb(None);

        let jump_to: *mut u8;
        unsafe {
            asm!("LDR {0}, ={fn_name}",
                out(reg) jump_to,
                fn_name = sym mmu_entry,
            );
        }
        dbgln!("relocate to pc: {}", jump_to);
        // Enable the MMU and turn on I-cache and D-cache
        cfg_match! {
            feature = "vm" => {
                SCTLR_EL2
                    .modify(SCTLR_EL2::M::Enable + SCTLR_EL2::C::Cacheable + SCTLR_EL2::I::Cacheable);
            }
            _ =>{
                SCTLR_EL1
                    .modify(SCTLR_EL1::M::Enable + SCTLR_EL1::C::Cacheable + SCTLR_EL1::I::Cacheable);
            }
        }

        isb(SY);
        unsafe {
            asm!(
                "MOV      x8,  {}",
                "BLR      x8",
                "B       .",
                in(reg) jump_to,
                options(nostack, noreturn)
            )
        }
    }

    #[inline(always)]
    pub fn is_mmu_enabled() -> bool {
        cfg_match! {
            feature = "vm" => {
                SCTLR_EL2.is_set(SCTLR_EL2::M)
            }
            _ =>{
                SCTLR_EL1.is_set(SCTLR_EL1::M)
            }
        }
    }
}
