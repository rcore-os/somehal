use core::ptr::NonNull;

use fdt_parser::FdtError;

use crate::mem::{PhysMemory, PhysMemoryArray};

#[link_boot::link_boot]
mod _m {
    use somehal_macros::println;

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
}
