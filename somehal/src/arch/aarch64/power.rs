use core::{fmt::Display, ops::Deref, ptr::NonNull};

use aarch64_cpu::asm::wfi;
use aarch64_cpu_ext::cache::{CacheOp, dcache_range};
use fdt_parser::Fdt;
use log::debug;
use pie_boot_if::BootInfo;
use smccc::{Hvc, Smc, psci};

use crate::{_start_secondary, boot_info, lazy_static::LazyStatic, println};

pub use smccc::psci::error::Error as PsciError;

#[unsafe(link_section = ".data")]
static METHOD: LazyStatic<Method> = LazyStatic::new();

#[derive(Debug, Clone, Copy)]
enum Method {
    Smc,
    Hvc,
}
impl From<&str> for Method {
    fn from(value: &str) -> Self {
        match value {
            "smc" => Method::Smc,
            "hvc" => Method::Hvc,
            _ => {
                panic!("Unsupported power method: {}", value);
            }
        }
    }
}
impl Display for Method {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Method::Smc => write!(f, "SMC"),
            Method::Hvc => write!(f, "HVC"),
        }
    }
}

pub(crate) fn init_by_fdt(fdt: Option<NonNull<u8>>) -> Option<()> {
    let fdt = fdt?;
    let fdt = Fdt::from_ptr(fdt).ok()?;
    let node = fdt
        .find_compatible(&["arm,psci-1.0", "arm,psci-0.2", "arm,psci"])
        .next()?;
    let method: Method = node.find_property("method")?.str().into();

    METHOD.init(method);
    println!("Power management method : {method}");
    Some(())
}

// Shutdown the system
pub fn shutdown() -> ! {
    match METHOD.deref() {
        Method::Smc => psci::system_off::<Smc>(),
        Method::Hvc => psci::system_off::<Hvc>(),
    }
    .unwrap();
    loop {
        wfi();
    }
}

/// Power on a CPU
pub fn cpu_on(cpu_id: u64, stack_top: u64) -> Result<(), PsciError> {
    unsafe {
        if super::UART_DEBUG == 0 {
            super::UART_DEBUG = boot_info()
                .debug_console
                .as_ref()
                .map(|p| p.base_virt as usize)
                .unwrap_or(0);
        }
    }

    let entry = secondary_entry_addr();

    let start = boot_info() as *const _ as usize;
    let size = core::mem::size_of::<BootInfo>();

    dcache_range(CacheOp::Clean, start, size);

    _cpu_on(cpu_id, entry as _, stack_top)
}

fn _cpu_on(cpu_id: u64, entry: u64, stack_top: u64) -> Result<(), smccc::psci::error::Error> {
    debug!(
        "[{}]Power on CPU {cpu_id:#x} at entry {entry:#x}, stack top {stack_top:#x}",
        METHOD.deref()
    );
    match METHOD.deref() {
        Method::Smc => psci::cpu_on::<Smc>(cpu_id, entry, stack_top)?,
        Method::Hvc => psci::cpu_on::<Hvc>(cpu_id, entry, stack_top)?,
    };
    Ok(())
}

/// secondary entry address
/// arg0 is stack top
fn secondary_entry_addr() -> usize {
    let ptr = _start_secondary as usize;
    ptr - boot_info().kcode_offset()
}
