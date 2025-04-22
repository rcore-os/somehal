fn main() {
    println!("cargo::rustc-check-cfg=cfg(fdt)");
    println!("cargo::rustc-check-cfg=cfg(early_debug)");
    println!("cargo::rustc-check-cfg=cfg(early_uart)");
    println!("cargo::rustc-check-cfg=cfg(hard_float)");

    let target = std::env::var("TARGET").unwrap();

    let mut need_fdt = false;
    let mut early_uart = false;

    if target.as_str() == "aarch64-unknown-none" {
        println!("cargo::rustc-cfg=hard_float");
    }

    let early_debug = std::env::var("CARGO_FEATURE_EARLY_DEBUG").is_ok();

    if target.contains("riscv") {
        need_fdt = true;
    }

    if target.contains("aarch64-") && early_debug {
        need_fdt = true;
        if early_debug {
            early_uart = true;
        }
    }

    if early_debug {
        println!("cargo::rustc-cfg=early_debug");
    }

    if need_fdt {
        println!("cargo::rustc-cfg=fdt");
    }

    if early_uart {
        println!("cargo::rustc-cfg=early_uart");
    }
}
