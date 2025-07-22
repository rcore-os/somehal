#![no_std]

use core::{fmt::Debug, mem::MaybeUninit, ptr::NonNull};

mod memregions;

pub use heapless::{String, Vec};
pub use memregions::*;

#[repr(C, align(64))]
#[derive(Clone)]
pub struct EarlyBootArgs {
    pub args: [usize; 4],
    pub virt_entry: *mut (),
    pub kimage_addr_lma: *mut (),
    pub kimage_addr_vma: *mut (),
    pub kcode_end: *mut (),
}

impl EarlyBootArgs {
    pub const fn new() -> Self {
        unsafe { MaybeUninit::zeroed().assume_init() }
    }
}

impl Default for EarlyBootArgs {
    fn default() -> Self {
        Self::new()
    }
}

#[repr(align(64))]
#[derive(Debug, Clone)]
pub struct BootInfo {
    /// CPU 硬件ID
    pub cpu_id: usize,
    /// 内核镜像物理地址
    pub kimage_start_lma: *mut u8,
    /// 内核镜像虚拟地址
    pub kimage_start_vma: *mut u8,
    /// 设备树物理地址
    pub fdt: Option<NonNull<u8>>,
    /// 页表开始物理地址
    pub pg_start: *mut u8,
    /// 内存区域
    pub memory_regions: MemoryRegions,
    /// 调试串口地址
    pub debug_console: Option<DebugConsole>,
    /// 可用内存起始地址
    pub free_memory_start: *mut u8,
}

unsafe impl Send for BootInfo {}
unsafe impl Sync for BootInfo {}

impl BootInfo {
    pub const fn new() -> Self {
        unsafe { MaybeUninit::zeroed().assume_init() }
    }

    pub fn kcode_offset(&self) -> usize {
        self.kimage_start_vma as usize - self.kimage_start_lma as usize
    }
}

impl Default for BootInfo {
    fn default() -> Self {
        Self::new()
    }
}

const COMPATIBLE_STRING_CAPACITY: usize = 32;
const COMPATIBLE_VEC_CAPACITY: usize = 3;

#[derive(Debug, Clone)]
pub struct DebugConsole {
    /// IO\MMIO 物理基地址
    pub base: *mut u8,
    pub compatibles: Vec<String<COMPATIBLE_STRING_CAPACITY>, COMPATIBLE_VEC_CAPACITY>,
}
