#![allow(unused)]

use core::fmt::Debug;

use crate::{PhysAddr, VirtAddr};

include!(concat!(env!("OUT_DIR"), "/constant.rs"));

const ADDR_BASE: usize = !((1 << ADDR_BITS) - 1);
const REGION_ONE: usize = (1 << ADDR_BITS) / 16;

pub const OFFSET_LINER: usize = ADDR_BASE + REGION_ONE * 8;
pub const STACK_TOP: usize = ADDR_BASE + REGION_ONE * 15;

static mut KCODE_VA_OFFSET: usize = 0;

pub const KERNEL_ADDR_SPACE_START: usize = ADDR_BASE;
pub const KERNEL_ADDR_SPACE_SIZE: usize = (1 << ADDR_BITS) - 1;

/// 设置内核代码的虚拟地址偏移量
///
/// # Safety
///
/// 确保 `offset` 是一个合法的虚拟地址偏移量
pub unsafe fn set_kcode_va_offset(offset: usize) {
    unsafe { KCODE_VA_OFFSET = offset };
}

pub fn kcode_offset() -> usize {
    unsafe { KCODE_VA_OFFSET }
}

bitflags::bitflags! {
    #[repr(transparent)]
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct AccessFlags: u8 {
        const Read = 1;
        const Write = 1 << 2;
        const Execute = 1 << 3;
        const LowerRead = 1 << 4;
        const LowerWrite = 1 << 5;
        const LowerExecute = 1 << 6;
    }
}

impl Debug for AccessFlags {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}{}{} |L: {}{}{}",
            if self.contains(AccessFlags::Read) {
                "R"
            } else {
                "-"
            },
            if self.contains(AccessFlags::Write) {
                "W"
            } else {
                "-"
            },
            if self.contains(AccessFlags::Execute) {
                "X"
            } else {
                "-"
            },
            if self.contains(AccessFlags::LowerRead) {
                "R"
            } else {
                "-"
            },
            if self.contains(AccessFlags::LowerWrite) {
                "W"
            } else {
                "-"
            },
            if self.contains(AccessFlags::LowerExecute) {
                "X"
            } else {
                "-"
            },
        )
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheConfig {
    Normal,
    Device,
    /// 需要强制写入主存的场景（如 DMA 缓冲区）
    WriteThrough,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MemConfig {
    pub access: AccessFlags,
    pub cache: CacheConfig,
}

impl Debug for MemConfig {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?} {:?}", self.access, self.cache)
    }
}

#[repr(C)]
#[derive(Clone)]
pub struct MemRegion {
    pub virt_start: VirtAddr,
    pub size: usize,
    pub phys_start: PhysAddr,
    pub name: &'static str,
    pub config: MemConfig,
    pub kind: MemRegionKind,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemRegionKind {
    Code,
    Stack,
    PerCpu,
    Memory,
    Device,
    Reserved,
}

pub fn region_phys_to_virt<'a, I: Iterator<Item = &'a MemRegion> + 'a>(
    regions: I,
    p: PhysAddr,
) -> VirtAddr {
    for region in regions {
        if p >= region.phys_start && p < region.phys_start + region.size {
            return region.virt_start + (p - region.phys_start);
        }
    }
    (p.raw() + OFFSET_LINER).into()
}

pub fn region_virt_to_phys<'a, I: Iterator<Item = &'a MemRegion> + 'a>(
    regions: I,
    v: VirtAddr,
) -> PhysAddr {
    for region in regions {
        let end = region.virt_start + region.size;
        if region.virt_start <= v && v < end {
            return region.phys_start + (v - region.virt_start);
        }
    }
    (v.raw() - OFFSET_LINER).into()
}
