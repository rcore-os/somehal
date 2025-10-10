//! 内核重定位支持
//!
//! 基于 Linux 6.11 arch/loongarch/kernel/relocate.c 实现

use core::arch::asm;

// 内核虚拟地址
const KERNEL_VADDR: usize = 0x9000000000000000;

// LoongArch 重定位类型 (参考 include/uapi/linux/elf.h)
const R_LARCH_NONE: usize = 0;
const R_LARCH_32: usize = 1;
const R_LARCH_64: usize = 2;
const R_LARCH_RELATIVE: usize = 3;
const R_LARCH_COPY: usize = 4;
const R_LARCH_JUMP_SLOT: usize = 5;
const R_LARCH_TLS_DTPMOD32: usize = 6;
const R_LARCH_TLS_DTPMOD64: usize = 7;
const R_LARCH_TLS_DTPREL32: usize = 8;
const R_LARCH_TLS_DTPREL64: usize = 9;
const R_LARCH_TLS_TPREL32: usize = 10;
const R_LARCH_TLS_TPREL64: usize = 11;

unsafe extern "C" {
    static __rela_dyn_begin: u8;
    static __rela_dyn_end: u8;
    static __la_abs_begin: u8;
    static __la_abs_end: u8;
    static __relr_dyn_begin: u8;
    static __relr_dyn_end: u8;
}

// RELA 重定位结构 (参考 include/uapi/linux/elf.h)
#[repr(C)]
struct Rela {
    r_offset: usize,
    r_info: usize,
    r_addend: isize,
}

// LoongArch 绝对地址条目结构 (参考 Linux arch/loongarch/include/asm/setup.h)
#[repr(C)]
#[allow(dead_code)]
struct RelaLaAbs {
    pc: isize,
    symvalue: isize,
}

/// 主要的重定位函数
/// 参考 Linux arch/loongarch/kernel/relocate.c:relocate_relative()
unsafe fn relocate_relative(reloc_offset: usize) {
    let rela_start = unsafe { &__rela_dyn_begin as *const u8 as usize };
    let rela_end = unsafe { &__rela_dyn_end as *const u8 as usize };

    if rela_start >= rela_end {
        return; // 没有重定位条目
    }

    let mut rela_ptr = rela_start as *const Rela;
    let rela_end_ptr = rela_end as *const Rela;

    while rela_ptr < rela_end_ptr {
        let rela = unsafe { &*rela_ptr };

        // 获取重定位类型
        let r_type = rela.r_info & 0xFF;

        // 只处理最基本的重定位类型
        match r_type {
            R_LARCH_RELATIVE => {
                // R_LARCH_RELATIVE: B + A (base address + addend)
                let addr = rela.r_offset.wrapping_add(reloc_offset);
                let value = (rela.r_addend as usize).wrapping_add(reloc_offset);
                unsafe {
                    core::ptr::write_volatile(addr as *mut usize, value);
                }
            }
            R_LARCH_64 => {
                // R_LARCH_64 entries hold absolute pointers that need the offset applied
                let addr = rela.r_offset.wrapping_add(reloc_offset) as *mut usize;
                unsafe {
                    let current = addr.read_volatile();
                    addr.write_volatile(current.wrapping_add(reloc_offset));
                }
            }
            _ => {}
        }

        rela_ptr = unsafe { rela_ptr.add(1) };
    }
}

/// RELR 重定位支持
/// 参考 Linux arch/loongarch/kernel/relocate.c 和 ARM64 实现
unsafe fn relocate_relr(reloc_offset: usize) {
    let relr_start = unsafe { &__relr_dyn_begin as *const u8 as usize };
    let relr_end = unsafe { &__relr_dyn_end as *const u8 as usize };

    if relr_start >= relr_end {
        return; // 没有 RELR 重定位条目
    }

    let mut addr: *mut usize = core::ptr::null_mut();
    let mut relr_ptr = relr_start as *const usize;
    let relr_end_ptr = relr_end as *const usize;

    while relr_ptr < relr_end_ptr {
        let relr = unsafe { *relr_ptr };

        if (relr & 1) == 0 {
            // 偶数条目：绝对地址
            addr = relr.wrapping_add(reloc_offset) as *mut usize;
            unsafe {
                *addr = (*addr).wrapping_add(reloc_offset);
                addr = addr.add(1);
            }
        } else {
            // 奇数条目：位图
            let mut bitmap = relr >> 1;
            let mut p = addr;
            while bitmap != 0 {
                if (bitmap & 1) != 0 {
                    unsafe {
                        *p = (*p).wrapping_add(reloc_offset);
                    }
                }
                unsafe {
                    p = p.add(1);
                }
                bitmap >>= 1;
            }
            addr = unsafe { addr.add(63) };
        }

        relr_ptr = unsafe { relr_ptr.add(1) };
    }
}

/// LoongArch 绝对地址重定位
/// 参考 Linux arch/loongarch/kernel/relocate.c:relocate_absolute()
#[allow(dead_code)]
unsafe fn relocate_absolute(reloc_offset: usize) {
    let la_abs_start = unsafe { &__la_abs_begin as *const u8 as usize };
    let la_abs_end = unsafe { &__la_abs_end as *const u8 as usize };

    if la_abs_start >= la_abs_end {
        return; // 没有 LoongArch 绝对地址重定位条目
    }

    let mut abs_ptr = la_abs_start as *const RelaLaAbs;
    let abs_end_ptr = la_abs_end as *const RelaLaAbs;

    while abs_ptr < abs_end_ptr {
        let abs_entry = unsafe { &*abs_ptr };

        // 计算需要重定位的指令地址
        let relocated_pc = (abs_entry.pc as usize).wrapping_add(reloc_offset);
        let symvalue = (abs_entry.symvalue as usize).wrapping_add(reloc_offset);

        // LoongArch 64位地址加载指令序列：
        // lu12i.w rd, %abs_hi20(symbol)
        // ori     rd, rd, %abs_lo12(symbol)
        // lu32i.d rd, %abs64_lo20(symbol)
        // lu52i.d rd, rd, %abs64_hi12(symbol)

        let insn_addr = relocated_pc as *mut u32;

        // 提取地址位段
        let lu12iw = (symvalue >> 12) & 0xfffff; // [31:12]
        let ori = symvalue & 0xfff; // [11:0]
        let lu32id = (symvalue >> 32) & 0xfffff; // [51:32]
        let lu52id = (symvalue >> 52) & 0xfff; // [63:52]

        unsafe {
            // lu12i.w: 位 [31:12] -> immediate [19:0]
            let insn0 = insn_addr.read_volatile();
            let new_insn0 = (insn0 & !0x1fffe0) | ((lu12iw as u32) << 5);
            insn_addr.write_volatile(new_insn0);

            // ori: 位 [11:0] -> immediate [11:0]
            let insn1 = insn_addr.add(1).read_volatile();
            let new_insn1 = (insn1 & !0xfff000) | ((ori as u32) << 10);
            insn_addr.add(1).write_volatile(new_insn1);

            // lu32i.d: 位 [51:32] -> immediate [19:0]
            let insn2 = insn_addr.add(2).read_volatile();
            let new_insn2 = (insn2 & !0x1fffe0) | ((lu32id as u32) << 5);
            insn_addr.add(2).write_volatile(new_insn2);

            // lu52i.d: 位 [63:52] -> immediate [11:0]
            let insn3 = insn_addr.add(3).read_volatile();
            let new_insn3 = (insn3 & !0xfff000) | ((lu52id as u32) << 10);
            insn_addr.add(3).write_volatile(new_insn3);
        }

        abs_ptr = unsafe { abs_ptr.add(1) };
    }
}

/// 主要的内核重定位函数
/// 参考 Linux arch/loongarch/kernel/relocate.c:relocate_kernel()
pub unsafe extern "C" fn relocate_kernel(reloc_offset: usize) {
    if reloc_offset == 0 {
        return; // 无需重定位
    }

    // 处理 RELA 重定位
    unsafe {
        relocate_relative(reloc_offset);
    }

    // 处理 RELR 重定位（压缩形式的 R_LARCH_RELATIVE 条目）
    unsafe {
        relocate_relr(reloc_offset);
    }

    // LoongArch 绝对地址重定位不是所有镜像都需要，先禁用避免潜在指令覆盖问题
    unsafe {
        relocate_absolute(reloc_offset);
    }
}

/// 早期重定位入口点
/// 在内核初始化的最早阶段调用
pub unsafe fn early_relocate() {
    let mut current_addr: usize;
    unsafe {
        asm!("la.pcrel $t0, .", "move {}, $t0", out(reg) current_addr);
    }

    // 检查是否需要重定位
    if current_addr != KERNEL_VADDR {
        // 计算重定位偏移
        let reloc_offset = current_addr.wrapping_sub(KERNEL_VADDR);

        unsafe {
            relocate_kernel(reloc_offset);
        }

        // 刷新指令与数据缓存，确保新指令立即生效
        unsafe {
            asm!("ibar 0");
            asm!("dbar 0");
        }
    }
}
