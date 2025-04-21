use core::ptr::NonNull;

use acpi::{madt::MadtEntry, *};
use spin::Mutex;

use crate::{once_static::OnceStatic, println};

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

pub fn cpu_list() {
    for ssdt in ACPI_TABLE.ssdts(){
    
    }

    let madt = ACPI_TABLE.find_table::<acpi::madt::Madt>().unwrap();
    for id in madt.get().entries().filter_map(|one| {
        if let MadtEntry::LocalApic(apic) = one {
            Some(apic.processor_id)
        } else {
            None
        }
    }) {
        println!("CPU {}", id);
    }
}
