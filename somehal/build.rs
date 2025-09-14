use kdef_pgtable::*;
use release_dep::{Config, get_release};
use serde::Deserialize;
use std::{io::Write, path::PathBuf, time::Duration};

#[derive(Deserialize)]
struct CargoToml {
    dependencies: Option<std::collections::HashMap<String, toml::Value>>,
    #[serde(rename = "target")]
    target_dependencies: Option<std::collections::HashMap<String, TargetDependencies>>,
}

#[derive(Deserialize)]
struct TargetDependencies {
    dependencies: Option<std::collections::HashMap<String, toml::Value>>,
}

fn main() {
    println!("cargo:rerun-if-changed=link.ld");
    println!("cargo:rerun-if-changed=link_base.ld");
    println!("cargo:rerun-if-changed=src/arch/loongarch64/link.ld");
    println!("cargo:rustc-link-search={}", out_dir().display());

    let target = std::env::var("TARGET").unwrap();

    println!("cargo::rustc-check-cfg=cfg(hard_float)");
    if target.as_str() == "aarch64-unknown-none" {
        println!("cargo::rustc-cfg=hard_float");
    }

    let kimage_vaddr = KIMAGE_VADDR;
    let page_size = PAGE_SIZE;

    let mut ld = include_str!("link_base.ld").to_string();

    macro_rules! set_var {
        ($v:ident) => {
            ld = ld.replace(concat!("{", stringify!($v), "}"), &format!("{:#x}", $v));
        };
    }

    set_var!(kimage_vaddr);
    set_var!(page_size);

    let ld_name_out = "pie_boot.x";

    let mut file = std::fs::File::create(out_dir().join(ld_name_out))
        .unwrap_or_else(|_| panic!("{ld_name_out} create failed"));
    file.write_all(ld.as_bytes())
        .unwrap_or_else(|_| panic!("{ld_name_out} write failed"));

    // 根据架构选择不同的链接脚本
    let ld = if target.contains("loongarch64") {
        include_str!("src/arch/loongarch64/link.ld").to_string()
    } else {
        include_str!("link.ld").to_string()
    };
    
    let ld_name_out = "somehal.x";
    let mut file = std::fs::File::create(out_dir().join(ld_name_out))
        .unwrap_or_else(|_| panic!("{ld_name_out} create failed"));
    file.write_all(ld.as_bytes())
        .unwrap_or_else(|_| panic!("{ld_name_out} write failed"));

    if std::env::var("TARGET").unwrap().contains("aarch64-") {
        aarch64_set_loader();
    }
}

fn out_dir() -> PathBuf {
    PathBuf::from(std::env::var("OUT_DIR").unwrap())
}

/// 从 Cargo.toml 中读取指定依赖的版本
fn get_dependency_version(package_name: &str) -> Option<String> {
    // 读取 Cargo.toml 文件
    let cargo_toml_content = std::fs::read_to_string("./Cargo.toml").ok()?;

    // 解析 TOML
    let cargo_toml: CargoToml = toml::from_str(&cargo_toml_content).ok()?;

    // 首先在普通依赖中查找
    if let Some(ref dependencies) = cargo_toml.dependencies
        && let Some(dep_value) = dependencies.get(package_name)
    {
        return extract_version_from_toml_value(dep_value);
    }

    // 在 target 特定的依赖中查找
    if let Some(ref target_deps) = cargo_toml.target_dependencies {
        for target_dep in target_deps.values() {
            if let Some(ref dependencies) = target_dep.dependencies
                && let Some(dep_value) = dependencies.get(package_name)
            {
                return extract_version_from_toml_value(dep_value);
            }
        }
    }

    println!("未找到依赖: {package_name}");
    None
}

/// 从 TOML 值中提取版本字符串
fn extract_version_from_toml_value(value: &toml::Value) -> Option<String> {
    match value {
        // 如果是简单的字符串版本，如 "0.2"
        toml::Value::String(version) => {
            println!("找到版本字符串: {version}");
            Some(version.clone())
        }
        // 如果是表格形式，如 { path = "../loader/pie-boot-loader-aarch64", version = "0.2" }
        toml::Value::Table(table) => {
            if let Some(toml::Value::String(version)) = table.get("version") {
                println!("找到版本: {version}");
                Some(version.clone())
            } else {
                println!("依赖表格中没有找到版本字段");
                None
            }
        }
        _ => {
            println!("未知的依赖格式");
            None
        }
    }
}

fn aarch64_set_loader() {
    // 首先尝试从 release 下载，如果失败则回退到本地构建
    let download_success = download_latest_release();

    if !download_success {
        // 回退到原来的本地构建逻辑
        build_loader_locally();
    }
}

fn download_latest_release() -> bool {
    let out_dir_str = out_dir().to_string_lossy().to_string();

    // 读取 Cargo metadata 获取依赖版本
    let package_version = get_dependency_version("pie-boot-loader-aarch64").unwrap_or_else(|| {
        println!("警告: 无法读取 pie-boot-loader-aarch64 依赖版本，使用默认版本 0.1.0");
        "0.1.0".to_string()
    });

    println!("使用依赖版本: {package_version}");

    let config = Config {
        package: "pie-boot-loader-aarch64",
        version: &package_version,
        download_dir: Some(&out_dir_str),
        repo: &[
            "https://gitee.com/zr233/somehal",
            "https://github.com/rcore-os/somehal",
        ],
        timeout: Some(Duration::from_secs(30)),
    };

    match get_release(config) {
        Ok(release_dep) => {
            println!("成功从 release 下载: {}", release_dep.name);
            println!("版本: {}", release_dep.version);
            println!("文件位置: {:?}", release_dep.binary);

            // 将下载的文件复制到预期位置
            let loader_dst = out_dir().join("loader.bin");
            let _ = std::fs::remove_file(&loader_dst);

            if let Err(e) = std::fs::copy(&release_dep.binary, &loader_dst) {
                println!("文件复制失败: {e}, 回退到本地构建");
                return false;
            }

            println!("成功复制到: {}", loader_dst.display());
            true
        }
        Err(e) => {
            println!("从 release 下载失败: {e}, 回退到本地构建");
            false
        }
    }
}

fn build_loader_locally() {
    let builder = bindeps_simple::Builder::new("pie-boot-loader-aarch64")
        .target("aarch64-unknown-none-softfloat")
        .env("RUSTFLAGS", "-C relocation-model=pic -Clink-args=-pie")
        .cargo_args(&["-Z", "build-std=core,alloc"]);

    let output = builder.build().unwrap();

    let loader_path = output.elf;
    let loader_dst = out_dir().join("loader.bin");

    let _ = std::fs::remove_file(&loader_dst);

    let status = std::process::Command::new("rust-objcopy")
        .args(["--strip-all", "-O", "binary"])
        .arg(&loader_path)
        .arg(loader_dst)
        .status()
        .expect("objcopy failed");

    assert!(status.success());

    println!("target dir: {}", target_dir().display());

    let _ = std::fs::remove_file(target_dir().join("loader.elf"));
    std::fs::copy(&loader_path, target_dir().join("loader.elf")).unwrap();
}

fn target_dir() -> PathBuf {
    PathBuf::from(std::env::var("OUT_DIR").unwrap())
        .ancestors()
        .nth(3)
        .unwrap()
        .into()
}
