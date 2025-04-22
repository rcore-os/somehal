use alloc::vec::Vec;

use core::{ptr::NonNull, slice::Iter};

use acpi::{
    madt::MadtEntry,
    platform::{Processor, ProcessorInfo, ProcessorState},
    *,
};
use spin::Mutex;

use crate::{mem::heap::GlobalHeap, once_static::OnceStatic, println};

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

pub fn cpu_list() -> impl Iterator<Item = CpuId> {
    let info = ACPI_TABLE.platform_info_in(GlobalHeap {}).unwrap();

    let mut ls = Vec::new_in(GlobalHeap {});

    if let Some(info) = info.processor_info {
        ls.push(info.boot_processor);

        for p in info.application_processors.iter() {
            ls.push(*p);
        }
    }

    ls.into_iter().filter_map(|one| {
        if matches!(one.state, ProcessorState::Disabled) {
            None
        } else {
            Some((one.processor_uid as usize).into())
        }
    })
}
