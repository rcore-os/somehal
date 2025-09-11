use core::{fmt::Display, ops::Deref, ptr::NonNull};

use aarch64_cpu::asm::wfi;
use aarch64_cpu_ext::cache::{CacheOp, dcache_all};
use fdt_parser::Fdt;
use log::debug;
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
    dcache_all(CacheOp::CleanAndInvalidate);
    let entry = secondary_entry_addr();
    // let entry = test_entry as usize - boot_info().kcode_offset();
    _cpu_on(cpu_id, entry as _, stack_top)
}

fn _cpu_on(cpu_id: u64, entry: u64, stack_top: u64) -> Result<(), smccc::psci::error::Error> {
    debug!(
        "[{}]Power on CPU {cpu_id:#x} at entry {entry:#x}, stack top {stack_top:#x}",
        METHOD.deref()
    );
    match METHOD.deref() {
        // Method::Smc => psci::cpu_on::<Smc>(cpu_id, entry, stack_top)?,
        // Method::Hvc => psci::cpu_on::<Hvc>(cpu_id, entry, stack_top)?,
        Method::Smc => cpu_on_smc(cpu_id, entry, stack_top)?,
        Method::Hvc => cpu_on_hvc(cpu_id, entry, stack_top)?,
    };
    Ok(())
}

#[unsafe(naked)]
unsafe extern "C" fn smc(cmd: u64, cpu_id: u64, entry: u64, stack_top: u64) -> i32 {
    core::arch::naked_asm!(
        "
        // Make SMC call
        smc #0
        
        // Return value is already in x0 (PSCI return code)
        ret
        "
    )
}

#[unsafe(naked)]
unsafe extern "C" fn hvc(cmd: u64, cpu_id: u64, entry: u64, stack_top: u64) -> i32 {
    core::arch::naked_asm!(
        "
        // Make HVC call
        hvc #0
        
        // Return value is already in x0 (PSCI return code)
        ret
        "
    )
}

fn cpu_on_smc(cpu_id: u64, entry: u64, stack_top: u64) -> Result<(), smccc::psci::error::Error> {
    let res = unsafe { smc(0xC4000003, cpu_id, entry, stack_top) };
    if res == 0 { Ok(()) } else { Err(res.into()) }
}

fn cpu_on_hvc(cpu_id: u64, entry: u64, stack_top: u64) -> Result<(), smccc::psci::error::Error> {
    let res = unsafe { hvc(0xC4000003, cpu_id, entry, stack_top) };
    if res == 0 { Ok(()) } else { Err(res.into()) }
}

/// PSCI CPU_ON using HVC call with naked_asm
#[unsafe(naked)]
unsafe extern "C" fn _cpu_on_hvc(cpu_id: u64, entry: u64, stack_top: u64) -> i32 {
    core::arch::naked_asm!(
        "
        // Setup PSCI CPU_ON parameters according to PSCI specification
        mov x1, x0              // x1 = target_cpu (first parameter -> cpu_id)
        mov x2, x1              // x2 = entry_point_address (second parameter -> entry)
        mov x3, x2              // x3 = context_id (third parameter -> stack_top)
        ldr x0, =0xC4000003    // x0 = PSCI CPU_ON function ID (0xC4000003)
        
        // Make HVC call
        hvc #0
        
        // Return value is already in x0 (PSCI return code)
        ret
        "
    )
}

/// secondary entry address
/// arg0 is stack top
fn secondary_entry_addr() -> usize {
    let ptr = _start_secondary as usize;
    ptr - boot_info().kcode_offset()
}

const UART: usize = 0x2800d000;
// const UART: usize = 0x9000000;

#[unsafe(naked)]
unsafe extern "C" fn test_entry() -> ! {
    core::arch::naked_asm!(
        "
        ldr x0, ={uart}            // 使用 ldr 指令加载常量地址
        mov w1, #0x41              // 'A' 字符的 ASCII 码
        str w1, [x0]               // 将字符写入 UARTDR 寄存器
        mov w1, {r}              // 
        str w1, [x0]               // 将字符写入 UARTDR 寄存器
        mov w1, {n}              // 
        str w1, [x0]               // 将字符写入 UARTDR 寄存器
        b .
    ",
        uart = const UART,
        r = const b'\r',
        n = const b'\n',
    )
}

pub fn cpu_on_test() {
    // let cpu_id = 0x1;
    // let stack_top = 0x47000000; // Example stack top address for the new CPU
    let cpu_id = 0x201;
    let stack_top = 0xf1000000; // Example stack top address for the new CPU
    let addr = test_entry as usize - boot_info().kcode_offset();
    debug!("Test CPU on addr: {addr:#x}");
    _cpu_on(cpu_id, addr as _, stack_top).unwrap();
}
