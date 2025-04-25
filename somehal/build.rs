use std::{
    fs::{self, exists, remove_dir_all},
    path::PathBuf,
    process,
};

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

    build_loader();
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
    fn gen_linker_script(&self, _config: &Config) {
        let script = "link.ld";

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

fn build_loader() {
    let outdir = out_dir();

    let src_dir = target_dir().join("loader_src");

    if !exists(&src_dir).unwrap() {
        process::Command::new("git")
            .args([
                "clone",
                "git@github.com:rcore-os/somehal.git",
                "-b",
                "test-bootloader",
            ])
            .arg(&src_dir)
            .status()
            .expect("git clone failed");
    }
    process::Command::new("git")
        .current_dir(&src_dir)
        .args(["pull", "--rebase"])
        .status()
        .expect("git pull failed");

    process::Command::new("cargo")
        .current_dir(&src_dir)
        .args([
            "build",
            "--release",
            "--target",
            &std::env::var("TARGET").unwrap(),
            "-p",
            "pie-boot",
            "--features",
            "early-debug",
        ])
        .status()
        .expect("cargo build failed");

    let loader_elf = outdir.join("loader.elf");
    let loader_bin = outdir.join("loader.bin");

    let _ = fs::copy(
        src_dir
            .join("target")
            .join(std::env::var("TARGET").unwrap())
            .join("release")
            .join("pie-boot"),
        &loader_elf,
    );

    process::Command::new("rust-objcopy")
        .arg(&loader_elf)
        .args(["--strip-all", "-O", "binary"])
        .arg(&loader_bin)
        .status()
        .unwrap();
}

fn target_dir() -> PathBuf {
    // 获取 OUT_DIR 环境变量
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR environment variable not set");

    // 将 OUT_DIR 转换为 Path 对象
    let out_path = PathBuf::from(out_dir);

    // 推导 target 文件夹的位置
    // OUT_DIR 的路径通常是：`<project_root>/target/<profile>/build/<crate_name>-<hash>/out`
    // 因此，我们可以通过多次调用 `parent()` 来找到 target 文件夹
    let target_dir = out_path
        .ancestors() // 遍历所有父目录
        .nth(5) // 第三个父目录就是 target 文件夹
        .expect("Failed to find target directory");

    println!("Target directory: {}", target_dir.display());

    target_dir.into()
}
