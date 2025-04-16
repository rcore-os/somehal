use std::{io::Write, path::PathBuf};

use quote::quote;

const MB: usize = 1024 * 1024;

// 2MiB stack size per hart
const DEFAULT_KERNEL_STACK_SIZE: usize = 2 * MB;

const DEFALUT_PAGE_SIZE: usize = 0x1000;

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

    let stack_size = if let Ok(s) = std::env::var("KERNEL_STACK_SIZE") {
        s.parse::<usize>()
            .expect("KERNEL_STACK_SIZE must be a number")
    } else {
        DEFAULT_KERNEL_STACK_SIZE
    };

    let page_size = if let Ok(s) = std::env::var("PAGE_SIZE") {
        s.parse::<usize>().expect("PAGE_SIZE must be a number")
    } else {
        DEFALUT_PAGE_SIZE
    };

    let const_content = quote! {
        pub const STACK_SIZE: usize = #stack_size;
        pub const PAGE_SIZE: usize = #page_size;
    };

    let mut file =
        std::fs::File::create(out_dir().join("constant.rs")).expect("constant.rs create failed");
    let syntax_tree = syn::parse2(const_content).unwrap();
    let formatted = prettyplease::unparse(&syntax_tree);
    file.write_all(formatted.as_bytes()).unwrap();
}

fn out_dir() -> PathBuf {
    PathBuf::from(std::env::var("OUT_DIR").unwrap())
}
