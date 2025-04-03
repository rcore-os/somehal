use kmem::space::CacheConfig;
pub use kmem::*;
use page::page_size;

use crate::consts::STACK_SIZE;
use somehal_macros::fn_link_section;

pub mod page;

#[derive(Debug, Clone)]
pub struct PhysMemory {
    pub addr: PhysAddr,
    pub size: usize,
}

#[link_boot::link_boot]
mod _m {
    use core::{
        alloc::Layout,
        ptr::slice_from_raw_parts_mut,
        sync::atomic::{AtomicUsize, Ordering},
    };

    use crate::dbgln;
    use kmem::space::{AccessFlags, MemConfig, OFFSET_LINER, STACK_TOP};

    use crate::{ArchIf, arch::Arch, once_static::OnceStatic, vec::ArrayVec};

    pub type PhysMemoryArray = ArrayVec<PhysMemory, 12>;
    static KCODE_VA_OFFSET: AtomicUsize = AtomicUsize::new(0);
    static mut CPU_COUNT: usize = 1;
    static MEM_REGIONS: OnceStatic<ArrayVec<MemRegion, 32>> = OnceStatic::new();
    static STACK_ALL: OnceStatic<PhysMemory> = OnceStatic::new();
    static PERCPU_ALL: OnceStatic<PhysMemory> = OnceStatic::new();
    static MEMORY_MAIN: OnceStatic<PhysMemory> = OnceStatic::new();

    #[repr(C)]
    #[derive(Clone)]
    pub struct MemRegion {
        pub virt_start: VirtAddr,
        pub size: usize,
        pub phys_start: PhysAddr,
        pub name: &'static str,
        pub config: MemConfig,
    }

    pub(crate) unsafe fn set_kcode_va_offset(offset: usize) {
        KCODE_VA_OFFSET.store(offset, Ordering::SeqCst);
    }

    fn bss_mut() -> &'static mut [u8] {
        unsafe extern "C" {
            fn __start_bss();
            fn __stop_bss();
        }
        unsafe {
            &mut *slice_from_raw_parts_mut(
                __start_bss as _,
                __stop_bss as usize - __start_bss as usize,
            )
        }
    }

    pub(crate) unsafe fn clean_bss() {
        bss_mut().fill(0);
    }

    pub(crate) fn entry_addr() -> usize {
        BootText().as_ptr() as usize
    }

    pub(crate) fn boot_stack_top() -> usize {
        unsafe extern "C" {
            fn __stack_bottom();
        }

        __stack_bottom as usize + STACK_SIZE
            - if Arch::is_mmu_enabled() {
                kcode_offset()
            } else {
                0
            }
    }

    pub(crate) fn kcode_offset() -> usize {
        KCODE_VA_OFFSET.load(Ordering::Relaxed)
    }

    fn link_section_end() -> PhysAddr {
        unsafe extern "C" {
            fn __stack_bottom();
        }
        (__stack_bottom as *mut u8 as usize).into()
    }

    pub(crate) fn stack_top_cpu0() -> PhysAddr {
        STACK_ALL.addr + STACK_SIZE
    }

    ///
    /// # Safety
    /// 只能在`mmu`开启前调用
    pub(crate) unsafe fn setup_memory_main(
        memories: impl Iterator<Item = PhysMemory>,
        cpu_count: usize,
    ) {
        detect_link_space();
        unsafe { CPU_COUNT = cpu_count };
        for m in memories {
            let mut phys_start = m.addr;
            let phys_raw = phys_start.raw();
            // let mut name = "memory    ";
            let size = m.size;
            let mut phys_end = phys_start + size;

            if phys_raw < link_section_end().raw() && link_section_end().raw() < phys_raw + m.size {
                // name = "mem main  ";
                phys_start = link_section_end();

                let stack_all_size = cpu_count * STACK_SIZE;

                phys_end = phys_end - stack_all_size;

                let stack_all = PhysMemory {
                    addr: phys_end,
                    size: stack_all_size,
                };

                unsafe {
                    (*STACK_ALL.get()).replace(stack_all);
                }

                unsafe {
                    (*MEMORY_MAIN.get()).replace(PhysMemory {
                        addr: phys_start,
                        size: phys_end.raw() - phys_start.raw(),
                    });
                }
            } else {
                mem_region_add(MemRegion {
                    virt_start: (phys_start.raw() + OFFSET_LINER).into(),
                    size,
                    phys_start,
                    name: "memory    ",
                    config: MemConfig {
                        access: AccessFlags::Read | AccessFlags::Write,
                        cache: CacheConfig::Normal,
                    },
                });
            }
        }

        let stack_start = STACK_ALL.addr + STACK_ALL.size - STACK_SIZE;

        mem_region_add(MemRegion {
            virt_start: (STACK_TOP - STACK_SIZE).into(),
            size: STACK_SIZE,
            phys_start: stack_start,
            name: "stack     ",
            config: MemConfig {
                access: AccessFlags::Read | AccessFlags::Write | AccessFlags::Execute,
                cache: CacheConfig::Normal,
            },
        });
    }

    pub(crate) fn setup_memory_regions(rsv: impl Iterator<Item = MemRegion>) {
        let percpu_size = percpu().len().align_up(page_size()) * unsafe { CPU_COUNT };

        let percpu_start =
            main_memory_alloc(Layout::from_size_align(percpu_size, page_size()).unwrap());

        let percpu_all = PhysMemory {
            addr: percpu_start,
            size: percpu_size,
        };

        unsafe {
            (*PERCPU_ALL.get()).replace(percpu_all);
        }
        mem_region_add(MemRegion {
            virt_start: (percpu().as_ptr() as usize + kcode_offset()).into(),
            size: percpu_size,
            phys_start: percpu_start,
            name: ".percpu   ",
            config: MemConfig {
                access: AccessFlags::Read | AccessFlags::Write | AccessFlags::Execute,
                cache: CacheConfig::Normal,
            },
        });

        mem_region_add(MemRegion {
            virt_start: (MEMORY_MAIN.addr.raw() + OFFSET_LINER).into(),
            size: MEMORY_MAIN.size,
            phys_start: MEMORY_MAIN.addr,
            name: "mem main  ",
            config: MemConfig {
                access: AccessFlags::Read | AccessFlags::Write,
                cache: CacheConfig::Normal,
            },
        });

        for r in rsv {
            mem_region_add(r);
        }
    }

    pub(crate) fn main_memory_alloc(layout: Layout) -> PhysAddr {
        unsafe {
            let end = MEMORY_MAIN.addr + MEMORY_MAIN.size;
            let ptr = MEMORY_MAIN.addr.align_up(layout.align());
            let start = ptr + layout.size();
            let size = end.raw() - start.raw();
            (*MEMORY_MAIN.get()).replace(PhysMemory { addr: start, size });
            ptr
        }
    }

    fn mem_region_add(mut region: MemRegion) {
        let size = region.size.align_up(page_size());
        region.size = size;

        dbgln!(
            "region {} : [{}, {}) -> [{}, {}) {}",
            region.name,
            region.virt_start.raw(),
            region.virt_start.raw() + region.size,
            region.phys_start.raw(),
            region.phys_start.raw() + region.size,
            if size == 0 { "skip empty" } else { "" }
        );

        if size == 0 {
            return;
        }

        if unsafe { (*MEM_REGIONS.get()).as_mut().unwrap() }
            .try_push(region)
            .is_err()
        {
            dbgln!("MemRegion is full");
            panic!();
        }
    }

    fn detect_link_space() {
        let regions = ArrayVec::new();
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
            ".text     ",
            text(),
            MemConfig {
                access: AccessFlags::Read | AccessFlags::Execute,
                cache: CacheConfig::Normal,
            },
        ));
        mem_region_add(link_section_to_kspace(
            ".rodata   ",
            rodata(),
            MemConfig {
                access: AccessFlags::Read | AccessFlags::Execute,
                cache: CacheConfig::Normal,
            },
        ));
        mem_region_add(link_section_to_kspace(
            ".data     ",
            data(),
            MemConfig {
                access: AccessFlags::Read | AccessFlags::Write | AccessFlags::Execute,
                cache: CacheConfig::Normal,
            },
        ));
        mem_region_add(link_section_to_kspace(
            ".bss      ",
            bss(),
            MemConfig {
                access: AccessFlags::Read | AccessFlags::Write | AccessFlags::Execute,
                cache: CacheConfig::Normal,
            },
        ));
    }

    /// `section`在mmu开启前是物理地址
    fn link_section_to_kspace(name: &'static str, section: &[u8], config: MemConfig) -> MemRegion {
        let phys_start = section.as_ptr() as usize;
        let virt_start = phys_start + kcode_offset();
        let size = section.len();
        MemRegion {
            virt_start: virt_start.into(),
            size,
            name,
            phys_start: phys_start.into(),
            config,
        }
    }
}

fn_link_section!(BootText);
fn_link_section!(BootData);
fn_link_section!(text);
fn_link_section!(data);
fn_link_section!(rodata);
fn_link_section!(bss);
fn_link_section!(percpu);

/// Returns an iterator over all physical memory regions.
pub fn memory_regions() -> impl Iterator<Item = MemRegion> {
    MEM_REGIONS.clone()
}
