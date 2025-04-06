use quote::quote;
use std::{io::Write, path::PathBuf};

const MB: usize = 1024 * 1024;

// 2MiB stack size per hart
const DEFAULT_KERNEL_STACK_SIZE: usize = 2 * MB;

// const ENTRY_VADDR: u64 = 0x40200000;
const ENTRY_VADDR: u64 = 0xffff_e000_0000_0000;
// const ENTRY_VADDR: u64 = 0x80200000;

fn main() {
    println!("cargo::rustc-link-arg=-Tlink.x");
    println!("cargo::rustc-link-arg=-no-pie");
    println!("cargo:rerun-if-changed=link.ld");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rustc-link-search={}", out_dir().display());

    println!("cargo::rustc-check-cfg=cfg(hard_float)");

    if std::env::var("TARGET").unwrap() == "aarch64-unknown-none" {
        println!("cargo::rustc-cfg=hard_float");
    }

    gen_const();

    let arch = Arch::default();

    println!("cargo::rustc-check-cfg=cfg(use_fdt)");
    if matches!(arch, Arch::Aarch64 | Arch::Riscv64) {
        println!("cargo::rustc-cfg=use_fdt");
    }

    arch.gen_linker_script();
}

#[derive(Debug)]
pub enum Arch {
    Aarch64,
    Riscv64,
    X86_64,
}

impl Default for Arch {
    fn default() -> Self {
        match std::env::var("CARGO_CFG_TARGET_ARCH").unwrap().as_str() {
            "aarch64" => Arch::Aarch64,
            "riscv64" => Arch::Riscv64,
            "x86_64" => Arch::X86_64,
            _ => unimplemented!(),
        }
    }
}

fn out_dir() -> PathBuf {
    PathBuf::from(std::env::var("OUT_DIR").unwrap())
}

impl Arch {
    fn gen_linker_script(&self) {
        let output_arch = if matches!(self, Arch::X86_64) {
            "i386:x86-64".to_string()
        } else if matches!(self, Arch::Riscv64) {
            "riscv".to_string() // OUTPUT_ARCH of both riscv32/riscv64 is "riscv"
        } else {
            format!("{:?}", self)
        };

        let ld_content = std::fs::read_to_string("link.ld").unwrap();
        let ld_content = ld_content.replace("%ARCH%", &output_arch);
        let ld_content = ld_content.replace("%KERNEL_VADDR%", &format!("{:#x}", ENTRY_VADDR));

        std::fs::write(out_dir().join("link.x"), ld_content).expect("link.x write failed");
    }
}

fn gen_const() {
    let entry_vaddr = ENTRY_VADDR as usize;

    let const_content = quote! {
        pub const KERNEL_STACK_SIZE: usize = #DEFAULT_KERNEL_STACK_SIZE;
        pub const KERNEL_ENTRY_VADDR: usize = #entry_vaddr;
    };

    let mut file =
        std::fs::File::create(out_dir().join("constant.rs")).expect("constant.rs create failed");
    let syntax_tree = syn::parse2(const_content).unwrap();
    let formatted = prettyplease::unparse(&syntax_tree);
    file.write_all(formatted.as_bytes()).unwrap();
}
