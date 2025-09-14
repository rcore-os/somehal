//! 内核重定位支持
//! 
//! 基于 Linux arch/loongarch/kernel/relocate.c 实现

use core::arch::asm;

// 内核虚拟地址
const KERNEL_VADDR: usize = 0x9000000000000000;

unsafe extern "C" {
    static __rela_dyn_begin: u8;
    static __rela_dyn_end: u8;
    static __la_abs_begin: u8;
    static __la_abs_end: u8;
}

pub unsafe extern "C" fn relocate_kernel() {
    // 获取重定位表地址
    let rela_start = unsafe { &__rela_dyn_begin as *const u8 as usize };
    let rela_end = unsafe { &__rela_dyn_end as *const u8 as usize };

    if rela_start >= rela_end {
        return; // 没有重定位条目
    }

    // 获取当前运行时地址
    let mut runtime_addr: usize;
    unsafe {
        asm!("la.pcrel $t0, .", "move {}, $t0", out(reg) runtime_addr);
    }

    // 计算重定位偏移
    let reloc_offset = runtime_addr.wrapping_sub(KERNEL_VADDR);

    if reloc_offset == 0 {
        return; // 无需重定位
    }

    // 处理 RELA 重定位
    unsafe {
        relocate_relative(rela_start, rela_end, reloc_offset);
    }

    // 处理 LoongArch 绝对地址重定位
    let la_abs_start = unsafe { &__la_abs_begin as *const u8 as usize };
    let la_abs_end = unsafe { &__la_abs_end as *const u8 as usize };
    
    if la_abs_start < la_abs_end {
        unsafe {
            relocate_absolute(la_abs_start, la_abs_end, reloc_offset);
        }
    }
}

// RELA 重定位结构
#[repr(C)]
struct Rela {
    r_offset: usize,
    r_info: usize,
    r_addend: isize,
}

unsafe fn relocate_relative(rela_start: usize, rela_end: usize, offset: usize) {
    let mut rela_ptr = rela_start as *const Rela;
    let rela_end_ptr = rela_end as *const Rela;

    while rela_ptr < rela_end_ptr {
        let rela = unsafe { &*rela_ptr };
        
        // R_LARCH_RELATIVE (值为 3)
        if (rela.r_info & 0xFF) == 3 {
            let addr = rela.r_offset + offset;
            let value = (rela.r_addend as usize).wrapping_add(offset);
            unsafe {
                core::ptr::write_volatile(addr as *mut usize, value);
            }
        }
        
        rela_ptr = unsafe { rela_ptr.add(1) };
    }
}

// LoongArch 绝对地址条目结构
#[repr(C)]
struct LaAbsEntry {
    pc: usize,
}

unsafe fn relocate_absolute(la_abs_start: usize, la_abs_end: usize, offset: usize) {
    let mut abs_ptr = la_abs_start as *const LaAbsEntry;
    let abs_end_ptr = la_abs_end as *const LaAbsEntry;

    while abs_ptr < abs_end_ptr {
        let abs_entry = unsafe { &*abs_ptr };
        
        // 计算需要重定位的指令地址
        let relocated_pc = abs_entry.pc + offset;
        
        // 读取指令并更新 
        let insn = unsafe { core::ptr::read_volatile(relocated_pc as *const u32) };
        
        // 提取立即数部分并加上偏移
        let imm = ((insn >> 5) & 0xFFFFF) as usize;
        let new_imm = (imm + offset) & 0xFFFFF;
        let new_insn = (insn & !0x1FFFE0) | ((new_imm as u32) << 5);
        
        unsafe {
            core::ptr::write_volatile(relocated_pc as *mut u32, new_insn);
        }
        
        abs_ptr = unsafe { abs_ptr.add(1) };
    }
}

pub unsafe fn early_relocate() {
    let mut current_addr: usize;
    unsafe {
        asm!("la.pcrel $t0, .", "move {}, $t0", out(reg) current_addr);
    }
    if current_addr != KERNEL_VADDR {
        unsafe {
            relocate_kernel();
        }
        // 指令缓存刷新
        unsafe {
            asm!("ibar 0");
        }
    }
}