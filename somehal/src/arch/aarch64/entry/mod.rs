use core::arch::{asm, naked_asm};

use aarch64_cpu::{asm::barrier, registers::*};
use pie_boot::BootInfo;

use super::debug;
use crate::{
    ArchIf, CpuOnArg,
    arch::{
        Arch, cache,
        paging::{set_kernel_table, set_user_table},
    },
    entry,
    mem::{page::new_mapped_table, setup_memory_regions, stack_top_phys, stack_top_virt},
    platform::*,
    printkv, println,
};

cfg_if::cfg_if! {
    if #[cfg(feature = "vm")] {
        mod el2;
        use el2::*;
    }else{
        mod el1;
        use el1::*;
    }
}

pub fn primary_entry(boot_info: BootInfo) {
    unsafe {
        cache::dcache_all(cache::DcacheOp::CleanAndInvalidate);
        println!();
        printkv!("Current EL", "{}", CurrentEL.read(CurrentEL::EL));

        entry::setup(boot_info);

        let sp = stack_top_phys(CpuIdx::primary());

        printkv!("Stack top", "{:?}", sp);

        // SP 移动到物理地址正确位置
        asm!(
            "MOV SP, {sp}",
            "B   {fix_sp}",
            sp = in(reg) sp.raw(),
            fix_sp = sym phys_sp_entry,
            options(noreturn, nostack),
        )
    }
}

fn phys_sp_entry() -> ! {
    println!("SP moved");

    setup_timer();

    setup_memory_regions(Arch::cpu_id(), cpu_list().unwrap());

    println!("Memory regions setup done!");

    let table = new_mapped_table(true);
    let sp = stack_top_virt(CpuIdx::primary());
    println!("Mov sp to {:?}", sp);

    debug::reloacte();

    set_kernel_table(table);
    set_user_table(table);

    unsafe {
        asm!(
            "MOV SP, {sp}",
            "B {f}",
            sp = in(reg) sp.raw(),
            f = sym crate::entry::entry_virt_and_liner,
            options(nostack, noreturn),
        );
    }
}

fn setup_timer() {
    #[cfg(not(feature = "vm"))]
    {
        CNTP_CTL_EL0.write(CNTP_CTL_EL0::ENABLE::SET);
        CNTP_TVAL_EL0.set(0);
    }
    #[cfg(feature = "vm")]
    {
        unsafe {
            // ENABLE, bit [0], Enables the timer.
            // * 0b0: Timer disabled.
            // * 0b1: Timer enabled.
            core::arch::asm!("msr CNTHP_CTL_EL2, {0:x}", in(reg) 0b1);
            core::arch::asm!("msr CNTHP_TVAL_EL2, {0:x}", in(reg) 0);
        }
    }
}
#[naked]
pub(crate) unsafe extern "C" fn secondary_entry() {
    unsafe {
        naked_asm!(
            "
        mov  sp, x0
        bl  {switch_el}
        mov  x0, sp
        bl  {init_mmu}
        bl  {enable_fp}
        mov  x0, sp
        bl  {sp_top}
        sub  x0, x0, {arg_size}
        mov  sp, x0
        LDR  x9, ={reloc}
        BLR  x9
        B    .
    ",
        switch_el = sym switch_to_elx,
        init_mmu = sym init_mmu,
        enable_fp = sym enable_fp,
        sp_top = sym get_sp_top,
        arg_size = const size_of::<CpuOnArg>(),
        reloc = sym relocate,
        )
    }
}

fn get_sp_top(arg: &CpuOnArg) -> usize {
    arg.stack_top_virt.raw()
}

unsafe fn enable_fp() {
    CPACR_EL1.write(CPACR_EL1::FPEN::TrapNothing);
    barrier::isb(barrier::SY);
}

fn relocate(arg: &CpuOnArg) {
    unsafe { asm!("msr daifset, #2") };
    CNTP_CTL_EL0.modify(CNTP_CTL_EL0::IMASK::SET);

    set_kernel_table(arg.page_table.raw().into());
    set_user_table(0usize.into());

    crate::init_secondary(arg)
}
