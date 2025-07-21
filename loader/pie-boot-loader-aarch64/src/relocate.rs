// ELF 重定位条目结构 (24 bytes)
#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct Elf64Rela {
    r_offset: u64, // 需要重定位的地址
    r_info: u64,   // 类型和符号索引
    r_addend: i64, // 加数值
}

impl Elf64Rela {
    #[inline]
    fn r_type_raw(&self) -> u32 {
        (self.r_info & 0xFFFFFFFF) as u32
    }
}

// 从链接器脚本导入符号
unsafe extern "C" {
    fn __rela_dyn_start();
    fn __rela_dyn_end();
}

// AArch64 重定位类型常量
const R_AARCH64_RELATIVE: u32 = 1027;
/// 计算加载偏移量 (实际地址 - 链接地址)
fn get_load_offset() -> i64 {
    sym_lma!(super::_start) as i64 - 0xF_0000_0000_i64
}

/// 应用 .rela.dyn 重定位
pub fn apply() {
    let load_offset = get_load_offset();

    let start = sym_lma!(__rela_dyn_start) as *mut Elf64Rela;
    let end = sym_lma!(__rela_dyn_end) as *const Elf64Rela;

    let num_entries = (end as usize - start as usize) / size_of::<Elf64Rela>();
    let relocations = unsafe { core::slice::from_raw_parts_mut(start, num_entries) };

    for reloc in relocations {
        if reloc.r_type_raw() == R_AARCH64_RELATIVE {
            let addr = (reloc.r_offset as i64 + load_offset) as usize as *mut usize;
            unsafe { *addr = (reloc.r_addend + load_offset) as usize };
        }
    }
}
