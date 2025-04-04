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
    use core::arch::naked_asm;

    use crate::{
        arch::debug,
        dbgln,
        mem::page::boot::{new_boot_table, new_boot_table2},
    };

    /// 参数为目标虚拟地址
    #[inline(always)]
    pub fn enable_mmu(stack_top: *mut u8, jump_to: *mut u8) -> ! {
        // let table = new_boot_table();
        let table = new_boot_table2();

        dbgln!("Set kernel table {}", table.raw());
        set_kernel_table(table);
        set_user_table(table);
        flush_tlb(None);

        dbgln!(
            "relocate to pc: {} stack: {}",
            jump_to as usize,
            stack_top as usize
        );
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
        debug::reloacte();
        jump(stack_top, jump_to)
    }

    #[naked]
    extern "C" fn jump(stack_top: *mut u8, jump_to: *mut u8) -> ! {
        unsafe {
            naked_asm!(
                "MOV      sp,  x0",
                "MOV      x8,  x1",
                "BLR      x8",
                "B       .",
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
