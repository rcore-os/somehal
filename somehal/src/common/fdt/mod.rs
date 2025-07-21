use core::ptr::NonNull;

use fdt_parser::Fdt;
use pie_boot_if::{MemoryRegion, MemoryRegionKind};

use crate::{common, lazy_static::LazyStatic};

static UART: LazyStatic<any_uart::Sender> = LazyStatic::new();

pub(crate) fn init_debugcon(fdt: Option<NonNull<u8>>) -> Option<()> {
    fn phys_to_virt(p: usize) -> *mut u8 {
        p as _
    }

    let uart = any_uart::init(fdt?, phys_to_virt)?;
    UART.init(uart.tx?);

    crate::early_debug::set_tx_fun(write_byte);
    Some(())
}

fn write_byte(b: u8) -> Result<(), crate::early_debug::TError> {
    unsafe {
        UART.edit(|tx| match tx.write(b) {
            Ok(_) => Ok(()),
            Err(e) => Err(match e {
                any_uart::Error::WouldBlock => crate::early_debug::TError::ReTry,
                any_uart::Error::Other(_e) => crate::early_debug::TError::Other,
            }),
        })
    }
}

pub fn find_rams(fdt: Option<NonNull<u8>>) -> Option<()> {
    let fdt = Fdt::from_ptr(fdt?).ok()?;
    for memory in fdt.memory() {
        for region in memory.regions() {
            let start = region.address as _;
            if region.size == 0 {
                continue; // Skip zero-sized regions
            }
            let v = MemoryRegion {
                start,
                end: start + region.size,
                kind: MemoryRegionKind::Ram,
            };

            common::mem::with_regions(|regions| regions.push(v).ok())?;
        }
    }

    for region in fdt.memory_reservation_block() {
        let start = region.address as _;
        let v = MemoryRegion {
            start,
            end: start + region.size,
            kind: MemoryRegionKind::Reserved,
        };
        common::mem::with_regions(|regions| regions.push(v).ok())?;
    }

    for node in fdt.reserved_memory() {
        if let Some(region) = node.reg().and_then(|mut r| r.next())
            && let Some(size) = region.size
        {
            let start = region.address as _;
            let v = MemoryRegion {
                start,
                end: start + size,
                kind: MemoryRegionKind::Reserved,
            };
            common::mem::with_regions(|regions| regions.push(v).ok())?;
        }
    }

    Some(())
}
