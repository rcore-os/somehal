use core::ops::Range;

use heapless::Vec;
use kdef_pgtable::{KLINER_OFFSET, PAGE_SIZE};
use num_align::{NumAlign, NumAssertAlign};
use pie_boot_if::{MemoryRegion, MemoryRegionKind};
use spin::Mutex;

pub use page_table_generic::PagingError;

use crate::{boot_info, common::entry::boot_info_edit};

mod stack;

pub(crate) use stack::init_percpu_stack;
pub use stack::{cpu_id_list, cpu_stack};

type MemoryRegionVec = Vec<MemoryRegion, 128>;

#[unsafe(link_section = ".data")]
static MEMORY_REGIONS: Mutex<MemoryRegionVec> = Mutex::new(Vec::new());

pub const fn page_size() -> usize {
    PAGE_SIZE
}

pub(crate) fn with_regions<F, R>(f: F) -> R
where
    F: FnOnce(&mut MemoryRegionVec) -> R,
{
    let mut regions = MEMORY_REGIONS.lock();
    f(&mut regions)
}

pub(crate) fn clean_bss() {
    unsafe extern "C" {
        fn __bss_start();
        fn __bss_stop();
    }
    unsafe {
        let bss = core::slice::from_raw_parts_mut(
            __bss_start as *mut u8,
            __bss_stop as usize - __bss_start as usize,
        );
        bss.fill(0);
    }
}

pub(crate) fn init_regions(args_regions: &[MemoryRegion]) {
    let mut regions = MEMORY_REGIONS.lock();
    regions
        .extend_from_slice(args_regions)
        .expect("Memory regions overflow");

    // 页对齐
    for region in regions.iter_mut() {
        if !region.end.is_aligned_to(page_size()) {
            let is_main = region.end == boot_info().free_memory_start as usize;
            region.end = region.end.align_up(page_size());
            if is_main {
                unsafe { boot_info_edit(|info| info.free_memory_start = region.end as _) };
            }
        }
    }

    // 添加内核前段预留
    add_kernel_reserved(&mut regions);

    // 从 RAM 中减去所有非 RAM 区域
    subtract_non_ram_from_ram(&mut regions);

    // 全局合并
    merge_regions(&mut regions);

    crate::println!("[MEM] Final memory regions: {:#?}", regions);
}

fn merge_regions(regions: &mut MemoryRegionVec) {
    if regions.is_empty() {
        return;
    }

    // 1. 按起始地址排序
    regions.as_mut_slice().sort_by_key(|r| r.start);

    // 2. 原地合并并检查冲突
    let mut write_idx = 0;
    for read_idx in 1..regions.len() {
        let next = regions[read_idx];
        let curr = &mut regions[write_idx];

        if next.start < curr.end {
            // 存在物理重叠
            if next.kind != curr.kind {
                crate::println!("FATAL: Memory regions of DIFFERENT kinds overlap!");
                crate::println!("  Region 1: {:#?}", curr);
                crate::println!("  Region 2: {:#?}", next);
                panic!("Memory regions of different kinds overlap");
            }
            // 类型相同：合并
            curr.end = curr.end.max(next.end);
        } else if next.start == curr.end && next.kind == curr.kind {
            // 物理相邻且类型相同：合并
            curr.end = next.end;
        } else {
            // 不重叠且不满足同类型相邻合并条件：移动写指针
            write_idx += 1;
            regions[write_idx] = next;
        }
    }

    regions.truncate(write_idx + 1);
}

fn find_main(regions: &MemoryRegionVec) -> Option<MemoryRegion> {
    let lma = boot_info().kimage_start_lma as usize;
    regions
        .iter()
        .find(|r| {
            let is_ram = matches!(r.kind, MemoryRegionKind::Ram);
            let in_range = r.start <= lma && r.end > lma;
            is_ram && in_range
        })
        .copied()
}

/// 添加内核前段预留区域
fn add_kernel_reserved(regions: &mut MemoryRegionVec) -> Option<()> {
    let mainmem = find_main(regions)?;
    let rsv_start = mainmem.start;

    unsafe extern "C" {
        fn _text();
    }
    let rsv_end = _text as usize - boot_info().kcode_offset();

    if rsv_start >= rsv_end {
        return Some(());
    }

    // 检查是否已被现有 Reserved 包含
    for r in regions.iter() {
        if matches!(r.kind, MemoryRegionKind::Reserved)
            && r.start <= rsv_start
            && r.end >= rsv_end
        {
            return Some(()); // 已包含，无需添加
        }
    }

    // 添加新的预留区域
    let _ = regions.push(MemoryRegion {
        kind: MemoryRegionKind::Reserved,
        start: rsv_start,
        end: rsv_end,
    });

    Some(())
}

/// 从所有 RAM 区域中减去非 RAM 区域（Reserved/Bootloader 等）
fn subtract_non_ram_from_ram(regions: &mut MemoryRegionVec) {
    // 1. 收集所有非 RAM 区域
    let mut non_ram: heapless::Vec<(usize, usize), 64> = regions
        .iter()
        .filter(|r| !matches!(r.kind, MemoryRegionKind::Ram))
        .map(|r| (r.start, r.end))
        .collect();

    if non_ram.is_empty() {
        return;
    }

    // 2. 对非 RAM 区域进行排序和合并（关键步骤！）
    non_ram.as_mut_slice().sort_by_key(|(s, _)| *s);
    let mut merged_non_ram: heapless::Vec<(usize, usize), 64> = heapless::Vec::new();
    let mut current = non_ram[0];
    
    for &(start, end) in non_ram.iter().skip(1) {
        if start <= current.1 {
            // 重叠或相邻，合并
            current.1 = current.1.max(end);
        } else {
            // 不连续，保存当前并开始新的
            let _ = merged_non_ram.push(current);
            current = (start, end);
        }
    }
    let _ = merged_non_ram.push(current);

    // 3. 收集所有 RAM 区域
    let ram_list: heapless::Vec<MemoryRegion, 8> = regions
        .iter()
        .filter(|r| matches!(r.kind, MemoryRegionKind::Ram))
        .copied()
        .collect();

    // 4. 删除所有旧 RAM 区域
    regions.retain(|r| !matches!(r.kind, MemoryRegionKind::Ram));

    // 5. 对每个 RAM 区域进行切分
    for ram in ram_list {
        let fragments = subtract_holes_from_range(ram.start, ram.end, &merged_non_ram);
        for (start, end) in fragments {
            let _ = regions.push(MemoryRegion {
                kind: MemoryRegionKind::Ram,
                start,
                end,
            });
        }
    }
}

/// 从一个连续范围中减去多个洞，返回剩余的碎片
fn subtract_holes_from_range(
    range_start: usize,
    range_end: usize,
    holes: &heapless::Vec<(usize, usize), 64>,
) -> heapless::Vec<(usize, usize), 16> {
    let mut fragments: heapless::Vec<(usize, usize), 16> = heapless::Vec::new();

    // 收集与当前范围重叠的洞
    let mut relevant_holes: heapless::Vec<(usize, usize), 32> = heapless::Vec::new();
    for &(hole_start, hole_end) in holes {
        if hole_start < range_end && hole_end > range_start {
            let actual_start = hole_start.max(range_start);
            let actual_end = hole_end.min(range_end);
            let _ = relevant_holes.push((actual_start, actual_end));
        }
    }

    if relevant_holes.is_empty() {
        // 没有洞，整个范围都是有效的
        let _ = fragments.push((range_start, range_end));
        return fragments;
    }

    // 按起始地址排序
    relevant_holes.as_mut_slice().sort_by_key(|(s, _)| *s);

    // 生成碎片
    let mut current = range_start;
    for (hole_start, hole_end) in relevant_holes {
        if current < hole_start {
            let _ = fragments.push((current, hole_start));
        }
        current = current.max(hole_end);
    }

    // 最后一段
    if current < range_end {
        let _ = fragments.push((current, range_end));
    }

    fragments
}

#[derive(Debug, Clone, Copy)]
pub enum CacheKind {
    Device,
    Normal,
    NoCache,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AccessKind {
    Read,
    ReadWrite,
    ReadExecute,
    ReadWriteExecute,
}
impl core::fmt::Debug for AccessKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            AccessKind::Read => write!(f, "R--"),
            AccessKind::ReadWrite => write!(f, "RW-"),
            AccessKind::ReadExecute => write!(f, "R-X"),
            AccessKind::ReadWriteExecute => write!(f, "RWX"),
        }
    }
}

pub struct MapRangeConfig {
    pub vaddr: *mut u8,
    pub paddr: usize,
    pub size: usize,
    pub name: &'static str,
    pub cache: CacheKind,
    pub access: AccessKind,
    pub cpu_share: bool,
}

fn region_ram_and_rsv() -> alloc::vec::Vec<MemoryRegion> {
    let src = MEMORY_REGIONS.lock();
    let mut out: alloc::vec::Vec<MemoryRegion> = src
        .iter()
        .filter(|r| {
            matches!(
                r.kind,
                MemoryRegionKind::Ram | MemoryRegionKind::Reserved
            )
        })
        .copied()
        .collect();

    if out.is_empty() {
        return out;
    }

    // 排序并合并
    out.sort_by_key(|r| r.start);

    let mut write_idx = 0;
    for read_idx in 1..out.len() {
        let next = out[read_idx];
        let curr = &mut out[write_idx];

        if next.start < curr.end {
            if next.kind != curr.kind {
                panic!("MMU map range conflict: {:?} overlaps with {:?}", curr, next);
            }
            curr.end = curr.end.max(next.end);
        } else if next.start == curr.end && next.kind == curr.kind {
            curr.end = next.end;
        } else {
            write_idx += 1;
            out[write_idx] = next;
        }
    }
    out.truncate(write_idx + 1);

    out
}

pub(crate) fn regions_to_map() -> alloc::vec::Vec<MapRangeConfig> {
    let mut map_ranges = alloc::vec::Vec::new();

    for region in region_ram_and_rsv() {
        map_ranges.push(MapRangeConfig {
            vaddr: phys_to_virt(region.start),
            paddr: region.start,
            size: region.end - region.start,
            name: "ram",
            cache: CacheKind::Normal,
            access: AccessKind::ReadWrite,
            cpu_share: true,
        });
    }

    if let Some(d) = &boot_info().debug_console {
        let start = d.base_phys.align_down(PAGE_SIZE);
        map_ranges.push(MapRangeConfig {
            vaddr: (start + KLINER_OFFSET) as *mut u8,
            paddr: start,
            size: PAGE_SIZE,
            name: "debug-con",
            cache: CacheKind::Device,
            access: AccessKind::ReadWrite,
            cpu_share: true,
        });
    }

    map_ranges.push(ld_range_to_map_config(
        "text",
        ld::text,
        true,
        AccessKind::ReadExecute,
    ));
    map_ranges.push(ld_range_to_map_config(
        "rodata",
        ld::rodata,
        true,
        AccessKind::ReadExecute,
    ));
    map_ranges.push(ld_range_to_map_config(
        "data",
        ld::data,
        true,
        AccessKind::ReadWriteExecute,
    ));
    map_ranges.push(ld_range_to_map_config(
        "bss",
        ld::bss,
        true,
        AccessKind::ReadWriteExecute,
    ));
    map_ranges.push(ld_range_to_map_config(
        "stack0",
        ld::stack0,
        false,
        AccessKind::ReadWriteExecute,
    ));

    let percpu_stack = stack::percpu_stack_range();
    if !percpu_stack.is_empty() {
        map_ranges.push(MapRangeConfig {
            vaddr: (percpu_stack.start + boot_info().kcode_offset()) as *mut u8,
            paddr: percpu_stack.start,
            size: percpu_stack.count(),
            name: "percpu-stack",
            cache: CacheKind::Normal,
            access: AccessKind::ReadWriteExecute,
            cpu_share: false,
        });
    }

    map_ranges
}
pub fn phys_to_virt(p: usize) -> *mut u8 {
    let v = if kimage_range_phys().contains(&p) {
        p + boot_info().kcode_offset()
    } else {
        // MMIO or other reserved regions
        p + KLINER_OFFSET
    };
    v as *mut u8
}
fn kimage_range_phys() -> Range<usize> {
    unsafe extern "C" {
        fn __kernel_code_end();
    }
    let start = boot_info().kimage_start_lma as usize;
    let end = __kernel_code_end as usize - boot_info().kcode_offset();
    start..end
}

fn ld_range_to_map_config(
    name: &'static str,
    ld: fn() -> Range<usize>,
    cpu_share: bool,
    access: AccessKind,
) -> MapRangeConfig {
    let range = ld();

    MapRangeConfig {
        vaddr: range.start as *mut u8,
        paddr: range.start - boot_info().kcode_offset(),
        size: range.count(),
        name,
        cache: CacheKind::Normal,
        access,
        cpu_share,
    }
}

mod ld {
    use super::*;
    macro_rules! ld_range {
        ($name:ident, $start:ident, $end:ident) => {
            pub fn $name() -> Range<usize> {
                unsafe extern "C" {
                    fn $start();
                    fn $end();
                }

                let start = $start as usize;
                let end = $end as usize;
                start..end
            }
        };
    }

    ld_range!(text, _stext, _etext);
    ld_range!(rodata, _srodata, _erodata);
    ld_range!(data, _sdata, _edata);
    ld_range!(stack0, __cpu0_stack, __cpu0_stack_top);
    ld_range!(bss, __bss_start, __bss_stop);
}

#[derive(Debug, Clone, Copy)]
pub struct PageTable {
    pub id: usize,
    pub addr: usize,
}
