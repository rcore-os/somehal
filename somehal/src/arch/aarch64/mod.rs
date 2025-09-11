use core::{
    arch::naked_asm,
    mem::{offset_of, size_of},
};

use kdef_pgtable::{KLINER_OFFSET, PAGE_SIZE};
#[cfg(not(feature = "hv"))]
use pie_boot_loader_aarch64::el1::{set_table, setup_sctlr, setup_table_regs};
#[cfg(feature = "hv")]
use pie_boot_loader_aarch64::el2::{set_table, setup_sctlr, setup_table_regs};

mod cache;
pub mod context;
pub mod mem;
pub mod power;
pub mod trap;

macro_rules! sym_lma {
    ($sym:expr) => {{
        #[allow(unused_unsafe)]
        unsafe{
            let out: usize;
            core::arch::asm!(
                "adrp {r}, {s}",
                "add  {r}, {r}, :lo12:{s}",
                r = out(reg) out,
                s = sym $sym,
            );
            out
        }
    }};
}

#[cfg_attr(feature = "hv", path = "el2.rs")]
#[cfg_attr(not(feature = "hv"), path = "el1.rs")]
mod el;

use crate::{BOOT_PT, boot_info, start_code};
use aarch64_cpu::{asm::barrier, registers::*};
use kasm_aarch64::{self as kasm, adr_l};
use pie_boot_loader_aarch64::EarlyBootArgs;

#[unsafe(link_section = ".data")]
static mut UART_DEBUG: usize = 0;
// static mut UART_DEBUG: usize = 0x2800d000;

const FLAG_LE: usize = 0b0;
const FLAG_PAGE_SIZE_4K: usize = 0b10;
const FLAG_ANY_MEM: usize = 0b1000;

#[unsafe(naked)]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".head.text")]
/// The header of the kernel.
///
/// # Safety
pub unsafe extern "C" fn _start() -> ! {
    naked_asm!(
        // code0/code1
        "nop",
        "bl {entry}",
        // text_offset
        ".quad 0",
        // image_size
        ".quad __kernel_load_end - _start",
        // flags
        ".quad {flags}",
        // Reserved fields
        ".quad 0",
        ".quad 0",
        ".quad 0",
        // magic - yes 0x644d5241 is the same as ASCII string "ARM\x64"
        ".ascii \"ARM\\x64\"",
        // Another reserved field at the end of the header
        ".byte 0, 0, 0, 0",
        flags = const FLAG_LE | FLAG_PAGE_SIZE_4K | FLAG_ANY_MEM,
        entry = sym primary_entry,
    )
}

#[start_code(naked)]
fn primary_entry() -> ! {
    naked_asm!(
    "
    bl  {preserve_boot_args}",
    adr_l!(x0, "{boot_args}"),
    adr_l!(x8, "{loader}"),
    "
    br   x8",
        preserve_boot_args = sym preserve_boot_args,
        boot_args = sym crate::BOOT_ARGS,
        loader = sym crate::loader::LOADER_BIN,
    )
}

#[start_code(naked)]
fn preserve_boot_args() {
    naked_asm!(
    adr_l!(x8, "{boot_args}"), // record the contents of
    "
	stp	x0,  x1, [x8]			// x0 .. x3 at kernel entry
	stp	x2,  x3, [x8, #16]

    LDR  x0,  ={virt_entry}
    str  x0,  [x8, {args_of_entry_vma}]",
    adr_l!(x0, "_start"),
    "
    str x0,  [x8, {args_of_kimage_addr_lma}]

    LDR  x0,  =_start
    str x0,  [x8, {args_of_kimage_addr_vma}]",

    adr_l!(x0, "__kernel_code_end"),
    "
    str x0,  [x8, {args_of_kcode_end}]

    // set EL
    mov x0, {el_value}              // Set target EL based on feature
    str x0,  [x8, {args_of_el}]

    LDR x0, ={kliner_offset}
    str x0,  [x8, {args_of_kliner_offset}]

    mov x0, {page_size}
    str x0,  [x8, {args_of_page_size}]

    mov x0, #1
    str x0,  [x8, {args_of_debug}]

	dmb	sy				// needed before dc ivac with
						// MMU off
    mov x0, x8                    
	add	x1, x0, {boot_arg_size}		
	b	{dcache_inval_poc}		// tail call
        ",
    boot_args = sym crate::BOOT_ARGS,
    virt_entry = sym switch_sp,
    args_of_entry_vma = const  offset_of!(EarlyBootArgs, virt_entry),
    args_of_kimage_addr_lma = const  offset_of!(EarlyBootArgs, kimage_addr_lma),
    args_of_kimage_addr_vma = const  offset_of!(EarlyBootArgs, kimage_addr_vma),
    args_of_kcode_end = const  offset_of!(EarlyBootArgs, kcode_end),
    args_of_el = const  offset_of!(EarlyBootArgs, el),
    el_value = const if cfg!(feature = "hv") { 2 } else { 1 },
    kliner_offset = const KLINER_OFFSET,
    args_of_kliner_offset = const offset_of!(EarlyBootArgs, kliner_offset),
    page_size = const PAGE_SIZE,
    args_of_page_size = const offset_of!(EarlyBootArgs, page_size),
    args_of_debug = const offset_of!(EarlyBootArgs, debug),
    dcache_inval_poc = sym cache::__dcache_inval_poc,
    boot_arg_size = const size_of::<EarlyBootArgs>()
    )
}

#[start_code(naked)]
pub fn _start_secondary(_stack_top: usize) -> ! {
    naked_asm!(
        "
        mrs     x19, mpidr_el1
        and     x19, x19, #0xffffff     // get current CPU id
        mov     x20, x0

        mov     sp, x20
        mov     x0, x20
        bl      {switch_to_elx}
        bl      {enable_fp}
        bl      {init_mmu} // return va_offset x0
        add     sp, sp, x0

        mov     x0, x19                 // call_secondary_main(cpu_id)
        ldr     x8, =__pie_boot_secondary
        blr     x8
        b      .",

        // t = sym test_print,
        switch_to_elx = sym el::switch_to_elx,
        init_mmu = sym init_mmu,
        enable_fp = sym enable_fp,
    )
}

#[start_code]
fn enable_fp() {
    CPACR_EL1.write(CPACR_EL1::FPEN::TrapNothing);
    barrier::isb(barrier::SY);
}

#[start_code]
fn init_mmu() -> usize {
    setup_table_regs();

    let addr = unsafe { BOOT_PT };
    set_table(addr);
    setup_sctlr();

    boot_info().kcode_offset()
}

#[unsafe(naked)]
unsafe extern "C" fn switch_sp(_args: usize) -> ! {
    naked_asm!(
        "
        adrp x8, __cpu0_stack_top
        add  x8, x8, :lo12:__cpu0_stack_top
        mov  sp, x8
        bl   {next}
        ",
        next =sym crate::common::entry::virt_entry,
    )
}

pub fn setup_exception_vectors() {
    trap::setup();
}
