#![no_std]
#![no_main]

use core::{
    arch::{asm, naked_asm},
    ptr::NonNull,
};

#[macro_use]
mod _macros;

mod cache;
mod console;
mod context;
#[cfg(feature = "console")]
mod debug;
mod reg;
#[cfg(el = "1")]
mod el1;
#[cfg(el = "2")]
mod el2;
mod lang_items;
mod mmu;
mod paging;
mod ram;
mod relocate;
mod staticcell;
mod trap;

use aarch64_cpu::{asm::barrier, registers::*};
#[cfg(el = "1")]
use el1::*;
#[cfg(el = "2")]
use el2::*;
use fdt_parser::Fdt;
use mmu::enable_mmu;
use pie_boot_if::EarlyBootArgs;
use staticcell::*;

pub use pie_boot_if::{BootInfo, DebugConsole, String, Vec};

#[unsafe(link_section = ".stack")]
static STACK: [u8; 0x8000] = [0; 0x8000];

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
        adr   x0, {stack}
        add   x0, x0, {stack_size}
        mov   sp, x0

        mov   x0, x19
        BL    {switch_to_elx}",

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
        stack = sym STACK,
        stack_size = const STACK.len(),
        switch_to_elx = sym switch_to_elx,
        entry = sym entry,
        offset = sym OFFSET,
        res = sym RETURN,
    )
}

fn entry(bootargs: &EarlyBootArgs) -> *mut () {
    enable_fp();
    unsafe {
        clean_bss();
        relocate::apply();
        cache::dcache_all(cache::DcacheOp::CleanAndInvalidate);

        let mut fdt = bootargs.args[0];
        OFFSET = bootargs.kimage_addr_vma as usize - bootargs.kimage_addr_lma as usize;
        ram::init(bootargs.kcode_end as _);

        #[cfg(feature = "console")]
        debug::fdt::init_debugcon(fdt as _);

        printkv!("fdt", "{fdt:#x}");

        trap::setup();

        asm!("msr daifset, #2");
        CNTP_CTL_EL0.modify(CNTP_CTL_EL0::IMASK::SET);

        fdt = save_fdt(fdt as _);

        printkv!("EL", "{}", CurrentEL.read(CurrentEL::EL));

        printkv!("_start", "{:p}", bootargs.kimage_addr_vma);

        let loader_at = loader_at();

        printkv!(
            "loader",
            "[{:p}, {:p})",
            loader_at,
            loader_at.add(loader_size())
        );
        enable_mmu(bootargs, fdt);
        let ret = RETURN.as_mut();

        ret.fdt = NonNull::new(fdt as _);
        ret.cpu_id = MPIDR_EL1.get() as usize & 0xFFFFFF;

        ret.kimage_start_lma = bootargs.kimage_addr_lma as _;
        ret.kimage_start_vma = bootargs.kimage_addr_vma as _;

        ret.memory_regions = ram::memory_regions().into();
        ret.free_memory_start = ram::current();
    }
    let jump = bootargs.virt_entry;
    printkv!("jump to", "{:p}", jump);
    jump
}

#[inline]
fn enable_fp() {
    CPACR_EL1.write(CPACR_EL1::FPEN::TrapNothing);
    barrier::isb(barrier::SY);
}

unsafe fn clean_bss() {
    concat!();
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
