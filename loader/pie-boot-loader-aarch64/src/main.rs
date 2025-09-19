#![no_std]
#![no_main]

use core::{
    arch::{asm, naked_asm},
    mem::offset_of,
    ptr::NonNull,
};

#[macro_use]
mod _macros;

mod cache;
mod console;
mod context;
mod debug;
pub mod def;
mod el1;
mod el2;
mod lang_items;
mod mmu;
mod paging;
mod ram;
mod reg;
mod relocate;
mod staticcell;
mod trap;

use aarch64_cpu::{asm::barrier, registers::*};

use crate::mmu::set_page_size;
use def::EarlyBootArgs;
use fdt_parser::Fdt;
use mmu::enable_mmu;
use pie_boot_if::BootInfo;
use staticcell::*;

pub(crate) static RETURN: StaticCell<BootInfo> = StaticCell::new(BootInfo::new());
static mut OFFSET: usize = 0;

/// The header of the kernel.
#[unsafe(no_mangle)]
#[unsafe(naked)]
#[unsafe(link_section = ".text.init")]
unsafe extern "C" fn _start(_args: &EarlyBootArgs) -> ! {
    naked_asm!(
        "
        mov   x19, x0

        ldr   x8, [x0, {args_of_stack_top_lma}]
        mov   sp, x8

        mov   x0, x19
        BL    {switch_to_target_el}",

        "mov   x0, x19",
        "BL     {entry}",
        "mov   x8,  x0",

        "
        adrp  x9,  {offset}
        ldr   x9,  [x9, :lo12:{offset}]
        add   sp, sp, x9

        adrp x0, {res}
        add  x0, x0, :lo12:{res}
        br   x8
        ",
        args_of_stack_top_lma = const offset_of!(EarlyBootArgs, stack_top_lma),
        switch_to_target_el = sym switch_to_target_el,
        entry = sym entry,
        offset = sym OFFSET,
        res = sym RETURN,
    )
}

/// Switch to the appropriate exception level based on EarlyBootArgs.el
fn switch_to_target_el(bootargs: &EarlyBootArgs) {
    let target_el = bootargs.el;
    let bootargs_ptr = bootargs as *const _ as usize;

    match target_el {
        1 => el1::switch_to_elx(bootargs_ptr),
        2 => el2::switch_to_elx(bootargs_ptr),
        _ => panic!("Unsupported exception level: {}", target_el),
    }
}

fn entry(bootargs: &EarlyBootArgs) -> *mut () {
    enable_fp();
    unsafe {
        clean_bss();

        relocate::apply();

        cache::dcache_all(cache::DcacheOp::CleanAndInvalidate);

        let mut fdt = bootargs.args[0];
        OFFSET = bootargs.kimage_addr_vma as usize - bootargs.kimage_addr_lma as usize;
        set_page_size(bootargs.page_size);
        ram::init(bootargs.kcode_end as _);

        if bootargs.debug() {
            debug::fdt::init_debugcon(fdt as _, bootargs.kliner_offset);
        }

        printkv!("fdt", "{fdt:#x}");

        trap::setup();

        asm!("msr daifset, #2");
        CNTP_CTL_EL0.modify(CNTP_CTL_EL0::IMASK::SET);

        fdt = save_fdt(fdt as _);

        printkv!("EL", "{}", CurrentEL.read(CurrentEL::EL));

        printkv!("_start", "{:p}", bootargs.kimage_addr_vma);
        printkv!("stack", "{:p}", bootargs.stack_top_vma);

        let loader_at = loader_at();

        printkv!(
            "loader",
            "[{:p}, {:p})",
            loader_at,
            loader_at.add(loader_size())
        );
        enable_mmu(bootargs, fdt);
        debug::relocate_uart(bootargs.kliner_offset);
        let ret = RETURN.as_mut();

        ret.fdt = NonNull::new(fdt as _);
        ret.cpu_id = MPIDR_EL1.get() as usize & 0xFFFFFF;

        ret.kimage_start_lma = bootargs.kimage_addr_lma as _;
        ret.kimage_start_vma = bootargs.kimage_addr_vma as _;

        ret.memory_regions = ram::memory_regions().into();
        ret.free_memory_start = ram::current();
        cache::flush_dcache_range(_stext as usize, _end as usize - _stext as usize);
    }
    let jump = bootargs.virt_entry;
    printkv!("jump to", "{:p}", jump);
    jump
}

unsafe extern "C" {
    fn _stext();
    fn _end();
}

#[inline]
fn enable_fp() {
    CPACR_EL1.write(CPACR_EL1::FPEN::TrapNothing);
    barrier::isb(barrier::SY);
}

unsafe fn clean_bss() {
    unsafe {
        let start = sym_lma_extern!(__start_boot_bss) as *mut u8;
        let end = sym_lma_extern!(__stop_boot_bss) as *mut u8;
        let len = end as usize - start as usize;
        for i in 0..len {
            start.add(i).write(0);
        }
    }
}

fn loader_size() -> usize {
    unsafe extern "C" {
        fn _stext();
        fn _end();
    }
    _end as usize - _stext as usize
}
fn loader_at() -> *mut u8 {
    let at;
    unsafe {
        asm!("
        adrp {0}, _stext
        add  {0}, {0}, :lo12:_stext
        ",
        out(reg) at
        );
    }
    at
}

fn save_fdt(fdt: *mut u8) -> usize {
    let ptr = match NonNull::new(fdt) {
        Some(v) => v,
        None => return 0,
    };
    unsafe {
        let fdt = Fdt::from_ptr(ptr).unwrap();
        let size = fdt.total_size();

        let dst = ram::alloc_phys(size, 64);

        let src = core::slice::from_raw_parts(ptr.as_ptr(), size);
        let dst_slice = core::slice::from_raw_parts_mut(dst, size);
        dst_slice.copy_from_slice(src);

        dst as usize
    }
}
