use core::alloc::Layout;

use heap::HEAP;
use heapless::Vec;
use kmem_region::region::{
    AccessFlags, CacheConfig, MemConfig, MemRegionKind, OFFSET_LINER, STACK_SIZE, STACK_TOP,
    kcode_offset, region_phys_to_virt, region_virt_to_phys,
};
pub use kmem_region::{region::MemRegion, *};
use somehal_macros::fn_link_section;

use crate::{
    once_static::OnceStatic,
    platform::{self, CpuId, CpuIdx},
    printkv, println,
};

pub(crate) mod heap;
pub(crate) mod main_memory;
pub mod page;
mod percpu;

use page::page_size;

#[derive(Debug, Clone)]
pub struct PhysMemory {
    pub addr: PhysAddr,
    pub size: usize,
}

pub type PhysMemoryArray = Vec<PhysMemory, 12>;

static MEMORY_MAIN: OnceStatic<PhysMemory> = OnceStatic::new();
static mut CPU_COUNT: usize = 1;
static MEM_REGIONS: OnceStatic<Vec<MemRegion, 32>> = OnceStatic::new();
static STACK_ALL: OnceStatic<PhysMemory> = OnceStatic::new();
// 除主CPU 外，其他CPU的独占Data
static PERCPU_OTHER_ALL: OnceStatic<PhysMemory> = OnceStatic::new();

pub fn cpu_idx() -> CpuIdx {
    unsafe { percpu::CPU_IDX }
}

pub fn cpu_id() -> CpuId {
    unsafe { percpu::CPU_ID }
}

pub(crate) fn stack_top_cpu0() -> PhysAddr {
    STACK_ALL.addr + STACK_SIZE
}

pub(crate) fn init_heap() {
    let mut h = HEAP.lock();
    if h.size() == 0 {
        let size = MEMORY_MAIN.size / 2;
        let start = MEMORY_MAIN.addr + size;
        printkv!("Tmp heap", "[{:?}, {:?})", start, start + size);
        unsafe {
            h.init(start.raw() as _, size);
        }
    }
}

pub(crate) fn setup_memory_main(
    reserved_memories: impl Iterator<Item = PhysMemory>,
    memories: impl Iterator<Item = PhysMemory>,
    cpu_count: usize,
) {
    detect_link_space();
    unsafe {
        CPU_COUNT = cpu_count;
    };

    let text_addr = text().as_ptr() as usize - kcode_offset();

    let mut main_memory = None;

    for m in memories {
        let phys_start = m.addr;
        let phys_raw = phys_start.raw();
        let size = m.size;

        if phys_raw < text_addr && text_addr < phys_raw + m.size {
            main_memory = Some(PhysMemory {
                addr: phys_start,
                size,
            });
        } else {
            mem_region_add(MemRegion {
                virt_start: (phys_start.raw() + OFFSET_LINER).into(),
                size,
                phys_start,
                name: "memory",
                config: MemConfig {
                    access: AccessFlags::Read | AccessFlags::Write,
                    cache: CacheConfig::Normal,
                },
                kind: MemRegionKind::Memory,
            });
        }
    }

    if let Some(mut main) = main_memory {
        let mut main_end = main.addr + main.size;

        for rsv in reserved_memories {
            let rsv_end = rsv.addr + rsv.size;

            if main.addr < rsv_end && rsv_end < main_end {
                main.addr = rsv_end.align_up(page_size());
            }
        }

        let stack_all_size = cpu_count * STACK_SIZE;

        main_end = main_end - stack_all_size;

        let stack_all = PhysMemory {
            addr: main_end,
            size: stack_all_size,
        };

        unsafe {
            (*STACK_ALL.get()).replace(stack_all);

            main_memory::init(main.addr, main_end);

            printkv!(
                "Found main memory",
                "[{:?}, {:?})",
                MEMORY_MAIN.addr,
                MEMORY_MAIN.addr + MEMORY_MAIN.size
            );
        }
    } else {
        println!("main memory not found!");
        panic!();
    }

    mem_region_add(MemRegion {
        virt_start: (STACK_TOP - STACK_SIZE).into(),
        size: STACK_SIZE,
        phys_start: stack_top_cpu0() - STACK_SIZE,
        name: "stack",
        config: MemConfig {
            access: AccessFlags::Read | AccessFlags::Write | AccessFlags::Execute,
            cache: CacheConfig::Normal,
        },
        kind: MemRegionKind::Stack,
    });

    init_heap();
}

pub(crate) fn setup_memory_regions(cpu0_id: CpuId, cpu_list: impl Iterator<Item = CpuId>) {
    let percpu_one_size = percpu().len().align_up(page_size());
    let percpu_cpu0_start = percpu().as_ptr() as usize - kcode_offset();

    let cpu_other_count = unsafe { CPU_COUNT - 1 };

    let percpu_all = if cpu_other_count > 0 {
        let percpu_other_all_size = percpu_one_size * cpu_other_count;

        let percpu_start = main_memory::alloc(
            Layout::from_size_align(percpu_other_all_size, page_size()).unwrap(),
        );

        PhysMemory {
            addr: percpu_start,
            size: percpu_other_all_size,
        }
    } else {
        PhysMemory {
            addr: 0usize.into(),
            size: 0,
        }
    };

    unsafe { (*PERCPU_OTHER_ALL.get()).replace(percpu_all) };
    percpu::init(cpu0_id, cpu_list);

    mem_region_add(MemRegion {
        virt_start: percpu().as_ptr().into(),
        size: percpu_one_size,
        phys_start: percpu_cpu0_start.into(),
        name: ".percpu",
        config: MemConfig {
            access: AccessFlags::Read | AccessFlags::Write | AccessFlags::Execute,
            cache: CacheConfig::Normal,
        },
        kind: MemRegionKind::PerCpu,
    });

    mem_region_add(MemRegion {
        virt_start: (MEMORY_MAIN.addr.raw() + OFFSET_LINER).into(),
        size: MEMORY_MAIN.size,
        phys_start: MEMORY_MAIN.addr,
        name: "mem main",
        config: MemConfig {
            access: AccessFlags::Read | AccessFlags::Write,
            cache: CacheConfig::Normal,
        },
        kind: MemRegionKind::Memory,
    });

    for r in platform::memory_regions() {
        mem_region_add(r);
    }
}

pub(crate) fn kernal_load_start_link_addr() -> usize {
    BootText().as_ptr() as _
}

fn mem_region_add(mut region: MemRegion) {
    let size = region.size.align_up(page_size());
    region.size = size;

    println!(
        "region {:<17}: [{:?}, {:?}) -> [{:?}, {:?}) {:?} {:?} {}",
        region.name,
        region.virt_start,
        region.virt_start + region.size,
        region.phys_start,
        region.phys_start + region.size,
        region.config,
        region.kind,
        if size == 0 { "skip empty" } else { "" }
    );

    if size == 0 {
        return;
    }

    if unsafe { (*MEM_REGIONS.get()).as_mut().unwrap() }
        .push(region)
        .is_err()
    {
        panic!();
    }
}

fn detect_link_space() {
    let regions = Vec::new();
    unsafe {
        (*MEM_REGIONS.get()).replace(regions);
    }

    mem_region_add(link_section_to_kspace(
        ".text.boot",
        BootText(),
        MemConfig {
            access: AccessFlags::Read | AccessFlags::Execute,
            cache: CacheConfig::Normal,
        },
    ));
    mem_region_add(link_section_to_kspace(
        ".data.boot",
        BootData(),
        MemConfig {
            access: AccessFlags::Read | AccessFlags::Write | AccessFlags::Execute,
            cache: CacheConfig::Normal,
        },
    ));
    mem_region_add(link_section_to_kspace(
        ".text",
        text(),
        MemConfig {
            access: AccessFlags::Read | AccessFlags::Execute,
            cache: CacheConfig::Normal,
        },
    ));
    mem_region_add(link_section_to_kspace(
        ".rodata",
        rodata(),
        MemConfig {
            access: AccessFlags::Read | AccessFlags::Execute,
            cache: CacheConfig::Normal,
        },
    ));
    mem_region_add(link_section_to_kspace(
        ".data",
        rwdata(),
        MemConfig {
            access: AccessFlags::Read | AccessFlags::Write | AccessFlags::Execute,
            cache: CacheConfig::Normal,
        },
    ));
    mem_region_add(link_section_to_kspace(
        ".bss",
        bss(),
        MemConfig {
            access: AccessFlags::Read | AccessFlags::Write | AccessFlags::Execute,
            cache: CacheConfig::Normal,
        },
    ));
}

/// `section`在mmu开启前是物理地址
fn link_section_to_kspace(name: &'static str, section: &[u8], config: MemConfig) -> MemRegion {
    let virt_start: VirtAddr = section.as_ptr().into();
    let phys_start = virt_start.raw() - kcode_offset();
    let size = section.len();
    MemRegion {
        virt_start,
        size,
        name,
        phys_start: phys_start.into(),
        config,
        kind: MemRegionKind::Code,
    }
}

fn_link_section!(BootText);
fn_link_section!(BootData);
fn_link_section!(text);
fn_link_section!(rodata);
fn_link_section!(bss);
fn_link_section!(percpu);

#[inline(always)]
fn rwdata() -> &'static [u8] {
    unsafe extern "C" {
        fn __srwdata();

        fn __erwdata();

    }
    unsafe {
        let start = __srwdata as usize;
        let stop = __erwdata as usize;
        core::slice::from_raw_parts(start as *const u8, stop - start)
    }
}

/// Returns an iterator over all physical memory regions.
pub fn memory_regions() -> impl Iterator<Item = MemRegion> {
    MEM_REGIONS.clone().into_iter()
}

pub fn phys_to_virt(p: PhysAddr) -> VirtAddr {
    region_phys_to_virt(MEM_REGIONS.iter(), p)
}

pub fn virt_to_phys(v: VirtAddr) -> PhysAddr {
    region_virt_to_phys(MEM_REGIONS.iter(), v)
}
