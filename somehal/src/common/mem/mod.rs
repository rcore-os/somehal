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

    for region in regions.iter_mut() {
        if !region.end.is_aligned_to(page_size()) {
            let is_main = region.end == boot_info().free_memory_start as usize;

            region.end = region.end.align_up(page_size());

            if is_main {
                unsafe { boot_info_edit(|info| info.free_memory_start = region.end as _) };
            }
        }
    }

    mainmem_start_rsv(&mut regions);
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

fn mainmem_start_rsv(regions: &mut MemoryRegionVec) -> Option<()> {
    let mainmem = find_main(regions)?;

    let mut start = mainmem.start;
    unsafe extern "C" {
        fn _text();
    }
    let mut end = _text as usize - boot_info().kcode_offset();

    // 收集需要移除的 reserved 区域的索引
    let mut indices_to_remove: heapless::Vec<usize, 16> = heapless::Vec::new();

    // 遍历现有的 reserved 区域，调整新区域的范围以排除重叠部分
    for (i, r) in regions.iter().enumerate() {
        if !matches!(r.kind, MemoryRegionKind::Reserved) {
            continue;
        }

        // 检查是否有重叠
        if !(end <= r.start || start >= r.end) {
            // 如果现有 reserved 区域完全包含了新区域，则无需添加
            if r.start <= start && r.end >= end {
                return Some(());
            }

            // 如果现有 reserved 区域完全在新区域中间，标记移除
            if r.start >= start && r.end <= end {
                let _ = indices_to_remove.push(i);
                continue;
            }

            // 如果现有 reserved 区域与新区域的开始部分重叠
            if r.start <= start && r.end > start && r.end < end {
                start = r.end;
            }

            // 如果现有 reserved 区域与新区域的结束部分重叠
            if r.start > start && r.start < end && r.end >= end {
                end = r.start;
            }
        }
    }

    // 从后往前移除标记的区域（避免索引变化问题）
    for &i in indices_to_remove.iter().rev() {
        regions.swap_remove(i);
    }

    // 检查调整后的区域是否仍然有效
    if start >= end {
        return Some(());
    }

    // 添加新的 reserved 区域
    let _ = regions.push(MemoryRegion {
        kind: MemoryRegionKind::Reserved,
        start,
        end,
    });

    Some(())
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
    let src = MEMORY_REGIONS.lock().to_vec();
    let mut out: alloc::vec::Vec<MemoryRegion> = alloc::vec::Vec::new();

    for region in src {
        // 只处理 RAM 和 Reserved 类型的区域
        if !matches!(
            region.kind,
            MemoryRegionKind::Ram | MemoryRegionKind::Reserved
        ) {
            continue;
        }

        let mut merged = false;

        // 尝试与现有区域合并
        for o in &mut out {
            // 检查是否有重叠或相邻
            if (o.start..o.end).contains(&region.start) ||
                (o.start..o.end).contains(&(region.end.saturating_sub(1))) ||
                (region.start..region.end).contains(&o.start) ||
                (region.start..region.end).contains(&(o.end.saturating_sub(1))) ||
                // 相邻区域
                o.end == region.start ||
                region.end == o.start
            {
                // 合并区域：扩展边界
                o.start = o.start.min(region.start);
                o.end = o.end.max(region.end);
                merged = true;
                break;
            }
        }

        // 如果没有合并，添加新区域
        if !merged {
            out.push(region);
        }
    }

    // 多轮合并，直到没有更多可合并的区域
    loop {
        let mut changed = false;
        let mut i = 0;

        while i < out.len() {
            let mut j = i + 1;
            while j < out.len() {
                if out[i].kind == out[j].kind
                    && (
                        // 检查重叠或相邻
                        (out[i].start..out[i].end).contains(&out[j].start)
                            || (out[i].start..out[i].end).contains(&(out[j].end.saturating_sub(1)))
                            || (out[j].start..out[j].end).contains(&out[i].start)
                            || (out[j].start..out[j].end).contains(&(out[i].end.saturating_sub(1)))
                            || out[i].end == out[j].start
                            || out[j].end == out[i].start
                    )
                {
                    // 合并区域
                    out[i].start = out[i].start.min(out[j].start);
                    out[i].end = out[i].end.max(out[j].end);
                    out.swap_remove(j);
                    changed = true;
                } else {
                    j += 1;
                }
            }
            i += 1;
        }

        if !changed {
            break;
        }
    }

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
