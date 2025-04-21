use std::path::PathBuf;

struct Config {
    addr_bits: usize,
}

fn main() {
    println!("cargo::rustc-link-arg=-Tlink.x");
    println!("cargo::rustc-link-arg=-no-pie");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rustc-link-search={}", out_dir().display());

    println!("cargo::rustc-check-cfg=cfg(addr_bits, values(\"39\", \"48\", \"57\", \"64\"))");
    println!("cargo::rustc-check-cfg=cfg(hard_float)");

    if std::env::var("TARGET").unwrap() == "aarch64-unknown-none" {
        println!("cargo::rustc-cfg=hard_float");
    }

    let mut config = Config { addr_bits: 48 };

    if std::env::var("CARGO_FEATURE_SV39").is_ok() {
        println!("cargo::rustc-cfg=addr_bits=\"39\"");
        config.addr_bits = 39;
    } else {
        println!("cargo::rustc-cfg=addr_bits=\"48\"");
    }

    let arch = Arch::default();

    println!("cargo::rustc-check-cfg=cfg(use_fdt)");
    if matches!(arch, Arch::Aarch64 | Arch::Riscv64) {
        println!("cargo::rustc-cfg=use_fdt");
    }
    println!("cargo::rustc-check-cfg=cfg(use_acpi)");
    if matches!(arch, Arch::X86_64) {
        println!("cargo::rustc-cfg=use_acpi");
    }

    arch.gen_linker_script(&config);
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
    fn gen_linker_script(&self, config: &Config) {
        let mut script = "link.ld";

        let output_arch = if matches!(self, Arch::X86_64) {
            // script = "src/arch/x86_64/link.ld";
            "i386:x86-64".to_string()
        } else if matches!(self, Arch::Riscv64) {
            "riscv".to_string() // OUTPUT_ARCH of both riscv32/riscv64 is "riscv"
        } else {
            format!("{:?}", self)
        };

        println!("cargo:rerun-if-changed={}", script);
        let ld_content = std::fs::read_to_string(script).unwrap();
        let ld_content = ld_content.replace("%ARCH%", &output_arch);

        std::fs::write(out_dir().join("link.x"), ld_content).expect("link.x write failed");
    }
}
