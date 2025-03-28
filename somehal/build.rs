use std::path::PathBuf;

// 2MiB stack size per hart
const DEFAULT_KERNEL_STACK_SIZE: usize = 2 * 1024 * 1024;

// const ENTRY_VADDR: u64 = 0x40200000;
const ENTRY_VADDR: u64 = 0xffff_e000_0000_0000;

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

        let ld_content =
            ld_content.replace("%STACK_SIZE%", &format!("{:#x}", DEFAULT_KERNEL_STACK_SIZE));
        std::fs::write(out_dir().join("link.x"), ld_content).expect("link.x write failed");
    }
}

fn gen_const() {
    let const_content = format!(
        r#"pub const KERNEL_STACK_SIZE: usize = {:#x};
            "#,
        DEFAULT_KERNEL_STACK_SIZE
    );

    std::fs::write(out_dir().join("constant.rs"), const_content).expect("const write failed");
}
