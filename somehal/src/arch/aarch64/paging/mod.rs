use core::arch::asm;

use aarch64_cpu::{asm::barrier::*, registers::*};

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
    use somehal_macros::println;

    use crate::{arch::debug, early_err, mem::page::boot::new_boot_table};

    /// 参数为目标虚拟地址
    #[inline(always)]
    pub fn enable_mmu(stack_top: *mut u8, jump_to: *mut u8) -> ! {
        let table = early_err!(new_boot_table());

        println!("Set kernel table {}", table.raw());
        set_kernel_table(table);
        set_user_table(table);
        flush_tlb(None);

        println!(
            "relocate to pc: {} stack: {}",
            jump_to as usize, stack_top as usize
        );
        unsafe {
            // Enable the MMU and turn on I-cache and D-cache
            cfg_match! {
                feature = "vm" => {

                }
                _ =>{
                    SCTLR_EL1
                        .modify(SCTLR_EL1::M::Enable + SCTLR_EL1::C::Cacheable + SCTLR_EL1::I::Cacheable);
                }
            }

            isb(SY);
            debug::reloacte();
            asm!(
                "MOV      sp,  {stack}",
                "MOV      x8,  {entry}",
                "BLR      x8",
                "B       .",
                stack = in(reg) stack_top as usize,
                entry = in(reg) jump_to as usize,
                options(nomem, nostack,noreturn)
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
