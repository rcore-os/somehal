#![allow(dead_code)]

use core::arch::asm;

use super::addrspace::VMLINUX_LOAD_ADDRESS;

macro_rules! sym_lma {
    ($sym:expr) => {{
        #[allow(unused_unsafe)]
        unsafe{
            let out: usize;
            core::arch::asm!(
                "la.pcrel    {r}, {s}",
                r = out(reg) out,
                s = sym $sym,
            );
            out
        }
    }};
}

// LoongArch 重定位类型 (参考 include/uapi/linux/elf.h)
const R_LARCH_NONE: usize = 0;
const R_LARCH_32: usize = 1;
const R_LARCH_64: usize = 2;
const R_LARCH_RELATIVE: u32 = 3;
const R_LARCH_COPY: usize = 4;
const R_LARCH_JUMP_SLOT: usize = 5;
const R_LARCH_TLS_DTPMOD32: usize = 6;
const R_LARCH_TLS_DTPMOD64: usize = 7;
const R_LARCH_TLS_DTPREL32: usize = 8;
const R_LARCH_TLS_DTPREL64: usize = 9;
const R_LARCH_TLS_TPREL32: usize = 10;
const R_LARCH_TLS_TPREL64: usize = 11;

unsafe extern "C" {
    fn _text();
    fn __rela_dyn_begin();
    fn __rela_dyn_end();
}

// RELA 重定位结构 (参考 include/uapi/linux/elf.h)
#[repr(C)]
struct Rela {
    r_offset: u64, // 需要重定位的地址
    r_info: u64,   // 类型和符号索引
    r_addend: i64, // 加数值
}

impl Rela {
    #[inline]
    fn r_type_raw(&self) -> u32 {
        (self.r_info & 0xFFFFFFFF) as u32
    }
}

// LoongArch 绝对地址条目结构 (参考 Linux arch/loongarch/include/asm/setup.h)
#[repr(C)]
#[allow(dead_code)]
struct RelaLaAbs {
    pc: isize,
    symvalue: isize,
}

/// 计算加载偏移量 (实际地址 - 链接地址)
fn get_load_offset() -> i64 {
    sym_lma!(_text) as i64 - VMLINUX_LOAD_ADDRESS as i64
}

/// 应用 .rela.dyn 重定位
pub fn apply() {
    let load_offset = get_load_offset();

    let start = sym_lma!(__rela_dyn_begin) as *mut Rela;
    let end = sym_lma!(__rela_dyn_end) as *const Rela;

    let num_entries = (end as usize - start as usize) / size_of::<Rela>();
    let relocations = unsafe { core::slice::from_raw_parts_mut(start, num_entries) };

    for reloc in relocations {
        if reloc.r_type_raw() == R_LARCH_RELATIVE {
            let addr = (reloc.r_offset as i64 + load_offset) as usize as *mut usize;
            unsafe { *addr = (reloc.r_addend + load_offset) as usize };
        }
    }
}

/// 早期重定位入口点
pub unsafe fn efi_relocate() {
    apply();

    // 刷新指令与数据缓存，确保重定位后的数据立即生效
    unsafe {
        asm!("ibar 0", options(nostack));
        asm!("dbar 0", options(nostack));
    }
}
