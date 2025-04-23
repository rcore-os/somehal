use alloc::vec::Vec;
use bit_field::BitField;
use kmem_region::region::MemRegion;

use core::ptr::NonNull;

use acpi::{platform::ProcessorState, *};

use crate::{
    _alloc::vec,
    handle_err,
    mem::{PhysMemory, heap::GlobalHeap},
    once_static::OnceStatic,
    println,
};

use super::CpuId;

#[derive(Clone)]
struct AcpiImpl;

impl AcpiHandler for AcpiImpl {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> acpi::PhysicalMapping<Self, T> {
        unsafe {
            acpi::PhysicalMapping::new(
                physical_address,
                NonNull::new(physical_address as _).unwrap(),
                size,
                size,
                AcpiImpl,
            )
        }
    }

    fn unmap_physical_region<T>(_region: &acpi::PhysicalMapping<Self, T>) {}
}

static ACPI_TABLE: OnceStatic<acpi::AcpiTables<AcpiImpl>> = OnceStatic::new();

pub fn check_acpi() -> Result<(), AcpiError> {
    unsafe {
        AcpiTables::search_for_rsdp_bios(AcpiImpl)?;
    }
    Ok(())
}

pub fn init() {
    unsafe {
        let acpi_table = AcpiTables::search_for_rsdp_bios(AcpiImpl).unwrap();
        println!("ACPI found!");
        ACPI_TABLE.init(acpi_table);
    }
}

pub fn cpu_count() -> usize {
    let mut count = 0;

    let madt = handle_err!(ACPI_TABLE.find_table::<madt::Madt>());

    for one in madt.get().entries() {
        match one {
            madt::MadtEntry::LocalApic(v) => {
                let is_disabled = !{ v.flags }.get_bit(0);
                if !is_disabled {
                    count += 1
                }
            }
            madt::MadtEntry::LocalX2Apic(v) => {
                let is_disabled = !{ v.flags }.get_bit(0);
                if !is_disabled {
                    count += 1
                }
            }
            _ => {}
        }
    }

    count
}

// pub fn cpu_list() -> impl Iterator<Item = CpuId> {
//     let madt = handle_err!(ACPI_TABLE.find_table::<madt::Madt>());

//     madt.get().entries().filter_map(|entry| match entry {
//         madt::MadtEntry::LocalApic(local_apic_entry) => {
//             Some((local_apic_entry.processor_id as usize).into())
//         }
//         madt::MadtEntry::LocalX2Apic(local_x2_apic_entry) => todo!(),
//         _ => None,
//     })
// }

pub(crate) fn memory_regions() -> vec::Vec<MemRegion> {
    vec![]
}
