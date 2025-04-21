use multiboot::information::{MemoryManagement, Multiboot};

use crate::{
    ArchIf,
    arch::{Arch, uart16550},
    mem::boot::set_kcode_va_offset,
    platform, println,
};

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __vma_relocate_entry(kcode_offset: usize, mbi: usize) -> ! {
    unsafe {
        set_kcode_va_offset(kcode_offset);
        uart16550::init();

        println!("\r\nMMU ready");

        platform::init();

        let mut memory = Memory {};

        let info = Multiboot::from_ptr(mbi as _, &mut memory).unwrap();

        println!("memory {:#x}", info.find_highest_address());

        if let Some(regions) = info.memory_regions() {
            for region in regions {
                println!(
                    "memory@{:#x} {:?} Mb {:?}",
                    region.base_address(),
                    region.length() / 1024 / 1024,
                    region.memory_type(),
                );
            }
        }

        platform::cpu_list();

        Arch::wait_for_event();
        unreachable!()
    }
}

struct Memory {}

impl MemoryManagement for Memory {
    unsafe fn paddr_to_slice(
        &self,
        addr: multiboot::information::PAddr,
        length: usize,
    ) -> Option<&'static [u8]> {
        Some(unsafe { core::slice::from_raw_parts(addr as usize as _, length) })
    }

    unsafe fn allocate(
        &mut self,
        length: usize,
    ) -> Option<(multiboot::information::PAddr, &mut [u8])> {
        todo!()
    }

    unsafe fn deallocate(&mut self, addr: multiboot::information::PAddr) {
        todo!()
    }
}
