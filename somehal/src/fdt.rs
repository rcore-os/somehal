use core::{
    alloc::Layout,
    ptr::{NonNull, slice_from_raw_parts_mut},
};

use fdt_parser::{Fdt, FdtError};
use kmem::IntAlign;

use crate::mem::{PhysMemory, PhysMemoryArray, main_memory_alloc, page::page_size};

#[link_boot::link_boot]
mod _m {
    use core::{
        ptr::null_mut,
        sync::atomic::{AtomicPtr, Ordering},
    };

    use crate::dbgln;
    use kmem::space::{AccessFlags, CacheConfig, MemConfig, OFFSET_LINER};

    use crate::mem::{MemRegion, kcode_offset};

    static FDT_ADDR: AtomicPtr<u8> = AtomicPtr::new(null_mut());

    pub fn find_memory(fdt: *mut u8) -> Result<PhysMemoryArray, FdtError<'static>> {
        let mut mems = PhysMemoryArray::new();

        let fdt = fdt_parser::Fdt::from_ptr(NonNull::new(fdt).ok_or(FdtError::BadPtr)?)?;

        for mem in fdt.memory() {
            for region in mem.regions() {
                if region.size == 0 {
                    continue;
                }

                if mems
                    .try_push(PhysMemory {
                        addr: (region.address as usize).into(),
                        size: region.size,
                    })
                    .is_err()
                {
                    dbgln!("too many phys memory regions");
                    panic!();
                };
            }
        }

        Ok(mems)
    }

    pub fn cpu_count() -> Result<usize, FdtError<'static>> {
        let fdt = get_fdt().ok_or(FdtError::BadPtr)?;
        let nodes = fdt.find_nodes("/cpus/cpu");
        Ok(nodes.count())
    }

    pub fn init_debugcon() -> Option<(any_uart::Uart, MemRegion)> {
        fn phys_to_virt(p: usize) -> *mut u8 {
            p as _
        }

        let fdt = get_fdt()?;
        let choson = fdt.chosen()?;
        let node = choson.debugcon()?;

        let uart = any_uart::Uart::new_by_fdt_node(&node, phys_to_virt)?;

        let reg = node.reg()?.next()?;
        let phys_start = reg.address as usize;

        Some((
            uart,
            MemRegion {
                virt_start: (phys_start + OFFSET_LINER).into(),
                size: page_size(),
                phys_start: phys_start.into(),
                name: "debug uart",
                config: MemConfig {
                    access: AccessFlags::Read | AccessFlags::Write,
                    cache: CacheConfig::Device,
                },
            },
        ))
    }
    pub(crate) unsafe fn set_fdt_ptr(fdt: *mut u8) {
        FDT_ADDR.store(fdt, Ordering::SeqCst);
    }

    fn fdt_ptr() -> *mut u8 {
        FDT_ADDR.load(Ordering::SeqCst)
    }

    fn get_fdt<'a>() -> Option<Fdt<'a>> {
        Fdt::from_ptr(NonNull::new(fdt_ptr())?).ok()
    }

    pub(crate) fn save_fdt() -> Result<MemRegion, &'static str> {
        let ptr_src = fdt_ptr();
        let fdt = Fdt::from_ptr(NonNull::new(ptr_src).ok_or("")?).map_err(|_| "")?;
        let size = fdt.total_size().align_up(page_size());

        let ptr_dst = main_memory_alloc(Layout::from_size_align(size, page_size()).map_err(|_| "")?)
            .raw() as *mut u8;

        unsafe {
            let src = &mut *slice_from_raw_parts_mut(ptr_src, size);
            let dst = &mut *slice_from_raw_parts_mut(ptr_dst, size);
            dst.copy_from_slice(src);

            FDT_ADDR.store(ptr_dst, Ordering::SeqCst);
        }

        Ok(MemRegion {
            virt_start: (ptr_dst as usize + kcode_offset()).into(),
            size,
            phys_start: (ptr_dst as usize).into(),
            name: "fdt data  ",
            config: MemConfig {
                access: AccessFlags::Read,
                cache: CacheConfig::Normal,
            },
        })
    }
}
