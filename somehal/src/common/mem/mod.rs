use heapless::Vec;
use pie_boot_if::{MemoryRegion, MemoryRegionKind};
use spin::Mutex;

use crate::boot_info;

mod phy_alloc;

type MemoryRegionVec = Vec<MemoryRegion, 128>;

static MEMORY_REGIONS: Mutex<MemoryRegionVec> = Mutex::new(Vec::new());

const STACK_SIZE: usize = 0x4_0000;

pub fn with_regions<F, R>(f: F) -> R
where
    F: FnOnce(&mut MemoryRegionVec) -> R,
{
    let mut regions = MEMORY_REGIONS.lock();
    f(&mut regions)
}

pub fn clean_bss() {
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

pub fn init_regions(args_regions: &[MemoryRegion]) {
    let mut regions = MEMORY_REGIONS.lock();
    regions
        .extend_from_slice(args_regions)
        .expect("Memory regions overflow");

    mainmem_start_rsv(&mut regions);
}

fn find_main(regions: &MemoryRegionVec) -> MemoryRegion {
    let lma = boot_info().kimage_start_lma as usize;
    *regions
        .iter()
        .find(|r| {
            let is_ram = matches!(r.kind, MemoryRegionKind::Ram);
            let in_range = r.start <= lma && r.end > lma;
            is_ram && in_range
        })
        .unwrap()
}

fn mainmem_start_rsv(regions: &mut MemoryRegionVec) -> Option<()> {
    let lma = boot_info().kimage_start_lma as usize;

    let mainmem = regions.iter().find(|r| {
        let is_ram = matches!(r.kind, MemoryRegionKind::Ram);
        let in_range = r.start <= lma && r.end > lma;
        is_ram && in_range
    })?;

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
