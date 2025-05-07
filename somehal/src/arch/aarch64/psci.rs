use core::error::Error;

use crate::{
    archif::CpuId,
    driver::{
        DriverGeneric, DriverResult,
        power::*,
        probe::{HardwareKind, ProbeDevInfo},
        register::*,
    },
    once_static::OnceStatic,
};

use alloc::{boxed::Box, format, vec::Vec};
use kmem_region::PhysAddr;
use log::{debug, error};
use smccc::{Hvc, Smc, psci};

static METHOD: OnceStatic<Method> = OnceStatic::new();

rdrive_macros::module_driver!(
    name: "ARM PSCI",
    kind: DriverKind::Power,
    probe_kinds: &[
        ProbeKind::Fdt {
            compatibles: &["arm,psci-1.0","arm,psci-0.2","arm,psci"],
            on_probe: probe
        }
    ]
);

#[derive(Debug, Clone, Copy)]
enum Method {
    Smc,
    Hvc,
}

impl TryFrom<&str> for Method {
    type Error = Box<dyn Error>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "smc" => Ok(Method::Smc),
            "hvc" => Ok(Method::Hvc),
            _ => Err(format!("method [{value}] not support").into()),
        }
    }
}

struct Psci {
    method: Method,
}

impl DriverGeneric for Psci {
    fn open(&mut self) -> DriverResult {
        Ok(())
    }

    fn close(&mut self) -> DriverResult {
        Ok(())
    }
}

impl Interface for Psci {
    fn shutdown(&mut self) {
        if let Err(e) = match self.method {
            Method::Smc => psci::system_off::<Smc>(),
            Method::Hvc => psci::system_off::<Hvc>(),
        } {
            error!("shutdown failed: {}", e);
        }
    }
}

fn probe(node: Node<'_>, dev: ProbeDevInfo) -> Result<Vec<HardwareKind>, Box<dyn Error>> {
    drop(dev);
    let method = node
        .find_property("method")
        .ok_or("fdt no method property")?
        .str();
    let method = Method::try_from(method)?;
    unsafe {
        METHOD.init(method);

        super::mp::init(cpu_on);
    }
    let dev = HardwareKind::Power(Box::new(Psci { method }));
    debug!("PCSI [{:?}]", method);
    Ok(alloc::vec![dev])
}

fn cpu_on(
    cpu_id: CpuId,
    entry: usize,
    stack_top: PhysAddr,
) -> Result<(), alloc::boxed::Box<dyn Error>> {
    let method = *METHOD;
    match method {
        Method::Smc => psci::cpu_on::<Smc>(cpu_id.raw() as _, entry as _, stack_top.raw() as _)?,
        Method::Hvc => psci::cpu_on::<Hvc>(cpu_id.raw() as _, entry as _, stack_top.raw() as _)?,
    };
    Ok(())
}
