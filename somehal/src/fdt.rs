use core::ptr::NonNull;

use fdt_parser::{Fdt, FdtError};

use crate::mem::{PhysMemory, PhysMemoryArray};

#[link_boot::link_boot]
mod _m {
    use kmem::space::{AccessFlags, CacheConfig, MemConfig, OFFSET_LINER};
    use somehal_macros::println;

    use crate::{ArchIf, arch::Arch, mem::MemRegion};

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
                    println!("too many phys memory regions");
                    panic!();
                };
            }
        }

        Ok(mems)
    }

    pub fn cpu_count(fdt: *mut u8) -> Result<usize, FdtError<'static>> {
        let fdt = fdt_parser::Fdt::from_ptr(NonNull::new(fdt).ok_or(FdtError::BadPtr)?)?;
        let nodes = fdt.find_nodes("/cpus/cpu");
        Ok(nodes.count())
    }

    fn phys_to_virt(p: usize) -> *mut u8 {
        if Arch::is_mmu_enabled() {
            return (p + OFFSET_LINER) as _;
        }
        p as _
    }

    pub fn init_debugcon(fdt: *mut u8) -> Option<(any_uart::Uart, MemRegion)> {
        let fdt = Fdt::from_ptr(NonNull::new(fdt)?).ok()?;
        let choson = fdt.chosen()?;
        let node = choson.debugcon()?;

        let uart = any_uart::Uart::new_by_fdt_node(&node, phys_to_virt)?;

        let reg = node.reg()?.next()?;
        let phys_start = reg.address as usize;

        Some((
            uart,
            MemRegion {
                virt_start: (phys_start + OFFSET_LINER).into(),
                size: Arch::page_size(),
                phys_start: phys_start.into(),
                name: "debug uart",
                config: MemConfig {
                    access: AccessFlags::Read | AccessFlags::Write,
                    cache: CacheConfig::NoCache,
                },
            },
        ))
    }
}
