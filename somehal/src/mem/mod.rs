use kmem::space::CacheConfig;
pub use kmem::*;
use somehal_macros::fn_link_section;

use crate::consts::STACK_SIZE;

#[link_boot::link_boot]
mod _m {
    use core::ptr::slice_from_raw_parts_mut;

    use kmem::space::{AccessFlags, MemConfig};
    use somehal_macros::println;

    use crate::{ArchIf, arch::Arch, once_static::OnceStatic, vec::ArrayVec};

    static mut KCODE_VA_OFFSET: usize = 0;
    static MEM_REGION_LINK: OnceStatic<ArrayVec<MemRegion, 16>> = OnceStatic::new();

    #[repr(C)]
    pub struct MemRegion {
        pub virt_start: VirtAddr,
        pub size: usize,
        pub phys_start: PhysAddr,
        pub name: &'static str,
        pub config: MemConfig,
    }

    pub(crate) unsafe fn set_kcode_va_offset(offset: usize) {
        unsafe { KCODE_VA_OFFSET = offset };
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
        unsafe extern "C" {
            fn __start_BootText();
        }

        __start_BootText as usize
    }

    pub(crate) fn stack_top() -> usize {
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
        unsafe { KCODE_VA_OFFSET }
    }

    /// # Safety
    /// 只能在`mmu`开启前调用
    pub(crate) unsafe fn detect_space_by_dtb(dtb: *mut u8) {
        detect_link_space();
        println!("detect link space");
    }
    fn mem_region_add(region: MemRegion) {
        println!(
            "region {} : [{}, {}) -> [{}, {}) {}",
            region.name,
            region.virt_start.raw(),
            region.virt_start.raw() + region.size,
            region.phys_start.raw(),
            region.phys_start.raw() + region.size,
            if region.size == 0 { "skip empty" } else { "" }
        );

        if region.size == 0 {
            return;
        }

        if unsafe { (*MEM_REGION_LINK.get()).as_mut().unwrap() }
            .try_push(region)
            .is_err()
        {
            println!("MemRegion is full");
            panic!();
        }
    }

    fn detect_link_space() {
        let regions = ArrayVec::new();
        unsafe {
            (*MEM_REGION_LINK.get()).replace(regions);
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
                access: AccessFlags::Read | AccessFlags::Write,
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
            ".data     ",
            data(),
            MemConfig {
                access: AccessFlags::Read | AccessFlags::Write,
                cache: CacheConfig::Normal,
            },
        ));
        mem_region_add(link_section_to_kspace(
            ".rodata   ",
            data(),
            MemConfig {
                access: AccessFlags::Read,
                cache: CacheConfig::Normal,
            },
        ));

        mem_region_add(link_section_to_kspace(
            ".bss      ",
            data(),
            MemConfig {
                access: AccessFlags::Read | AccessFlags::Write,
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
