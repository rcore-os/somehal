use futures::future::select_all;
use kdef_pgtable::*;
use serde::Deserialize;
use std::{io::Write, path::PathBuf, time::Duration};
use tokio::time::timeout;

const REPO_LIST: &[&str] = &[
    "https://gitee.com/zr233/somehal",
    "https://github.com/rcore-os/somehal",
];

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    draft: bool,
    prerelease: bool,
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

#[derive(Debug, Deserialize)]
struct GiteeRelease {
    tag_name: String,
    #[serde(default)]
    draft: bool, // Gitee API 可能没有这个字段，使用默认值 false
    prerelease: bool,
    assets: Vec<GiteeAsset>,
}

#[derive(Debug, Deserialize)]
struct GiteeAsset {
    name: String,
    browser_download_url: String,
}

#[derive(Debug)]
struct UnifiedRelease {
    tag_name: String,
    draft: bool,
    prerelease: bool,
    loader_asset_url: Option<String>,
}

fn main() {
    println!("cargo:rerun-if-changed=link.ld");
    println!("cargo:rerun-if-changed=link_base.ld");
    println!("cargo:rustc-link-search={}", out_dir().display());

    let repo_base = test_repos_sync().expect("no available repo");

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

    let ld = include_str!("link.ld").to_string();
    let ld_name_out = "somehal.x";
    let mut file = std::fs::File::create(out_dir().join(ld_name_out))
        .unwrap_or_else(|_| panic!("{ld_name_out} create failed"));
    file.write_all(ld.as_bytes())
        .unwrap_or_else(|_| panic!("{ld_name_out} write failed"));

    if std::env::var("TARGET").unwrap().contains("aarch64-") {
        aarch64_set_loader(&repo_base);
    }
}

fn out_dir() -> PathBuf {
    PathBuf::from(std::env::var("OUT_DIR").unwrap())
}

fn aarch64_set_loader(repo_base: &str) {
    // 首先尝试从 release 下载，如果失败则回退到本地构建
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("创建 tokio runtime 失败");

    let download_success = rt.block_on(async {
        match download_latest_release(repo_base).await {
            Ok(()) => {
                println!("成功从 release 下载 pie-boot-loader-aarch64.bin");
                true
            }
            Err(e) => {
                println!("从 release 下载失败: {e}, 回退到本地构建");
                false
            }
        }
    });

    if !download_success {
        // 回退到原来的本地构建逻辑
        build_loader_locally();
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

/// 异步测试仓库连接速度，返回最快的仓库URL
async fn test_fastest_repo() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    println!("开始测试仓库连接速度...");

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    // 为每个仓库创建异步测试任务
    let mut futures = Vec::new();

    for (index, repo_url) in REPO_LIST.iter().enumerate() {
        let client = client.clone();
        let url = repo_url.to_string();

        let future = async move {
            let start = std::time::Instant::now();

            // 尝试连接仓库
            match timeout(Duration::from_secs(5), client.head(&url).send()).await {
                Ok(Ok(response)) => {
                    let elapsed = start.elapsed();
                    if response.status().is_success() || response.status().as_u16() == 302 {
                        println!("仓库 {url} 连接成功，耗时: {elapsed:?}");
                        Ok((index, url, elapsed))
                    } else {
                        Err(format!("仓库 {} 返回状态码: {}", url, response.status()))
                    }
                }
                Ok(Err(e)) => Err(format!("仓库 {url} 连接失败: {e}")),
                Err(_) => Err(format!("仓库 {url} 连接超时")),
            }
        };

        futures.push(Box::pin(future));
    }

    // 等待第一个成功的连接
    if futures.is_empty() {
        return Err("没有可测试的仓库".into());
    }

    let (result, _index, _remaining) = select_all(futures).await;

    match result {
        Ok((repo_index, fastest_url, elapsed)) => {
            println!("最快的仓库是: {fastest_url} (索引: {repo_index}, 耗时: {elapsed:?})");

            Ok(fastest_url)
        }
        Err(e) => Err(e.into()),
    }
}

/// 同步包装器，用于在 main 函数中调用异步测试
fn test_repos_sync() -> Option<String> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("创建 tokio runtime 失败");

    match rt.block_on(test_fastest_repo()) {
        Ok(fastest_repo) => {
            println!("选择的仓库: {fastest_repo}");
            Some(fastest_repo)
        }
        Err(e) => {
            println!("仓库测试失败: {e}");
            // 回退到默认仓库
            Some(REPO_LIST[0].to_string())
        }
    }
}

/// 下载最新的 pie-boot-loader-aarch64.bin release
async fn download_latest_release(
    repo_base: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    let releases = if repo_base.contains("github.com") {
        let github_releases = fetch_github_releases(&client, repo_base).await?;
        convert_github_releases(github_releases)
    } else if repo_base.contains("gitee.com") {
        let gitee_releases = fetch_gitee_releases(&client, repo_base).await?;
        convert_gitee_releases(gitee_releases)
    } else {
        return Err("不支持的仓库类型".into());
    };

    if releases.is_empty() {
        return Err("没有找到任何 release".into());
    }

    // 找到包含 pie-boot-loader-aarch64.bin 的最新稳定版本
    let latest_release = find_latest_stable_release(releases)?;

    println!("找到最新版本: {}", latest_release.tag_name);

    // 下载文件
    download_loader_bin(&client, &latest_release).await?;

    Ok(())
}

/// 获取 GitHub releases
async fn fetch_github_releases(
    client: &reqwest::Client,
    repo_base: &str,
) -> Result<Vec<GitHubRelease>, Box<dyn std::error::Error + Send + Sync>> {
    // 从URL中提取owner和repo名称
    let (owner, repo) = extract_owner_repo(repo_base)?;
    let url = format!("https://api.github.com/repos/{owner}/{repo}/releases");
    println!("正在获取 GitHub releases: {url}");

    let response = client
        .get(&url)
        .header("User-Agent", "somehal-build-script")
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("GitHub API 请求失败: {}", response.status()).into());
    }

    let releases: Vec<GitHubRelease> = response.json().await?;
    println!("获取到 {} 个 GitHub releases", releases.len());

    Ok(releases)
}

/// 获取 Gitee releases
async fn fetch_gitee_releases(
    client: &reqwest::Client,
    repo_base: &str,
) -> Result<Vec<GiteeRelease>, Box<dyn std::error::Error + Send + Sync>> {
    // 从URL中提取owner和repo名称
    let (owner, repo) = extract_owner_repo(repo_base)?;
    let url = format!("https://gitee.com/api/v5/repos/{owner}/{repo}/releases");
    println!("正在获取 Gitee releases: {url}");

    let response = client.get(&url).send().await?;

    if !response.status().is_success() {
        return Err(format!("Gitee API 请求失败: {}", response.status()).into());
    }

    let releases: Vec<GiteeRelease> = response.json().await?;
    println!("获取到 {} 个 Gitee releases", releases.len());

    Ok(releases)
}

/// 从仓库URL中提取owner和repo名称
fn extract_owner_repo(
    repo_url: &str,
) -> Result<(&str, &str), Box<dyn std::error::Error + Send + Sync>> {
    // 移除协议前缀和可能的.git后缀
    let url = repo_url
        .strip_prefix("https://")
        .or_else(|| repo_url.strip_prefix("http://"))
        .unwrap_or(repo_url)
        .strip_suffix(".git")
        .unwrap_or(
            repo_url
                .strip_prefix("https://")
                .or_else(|| repo_url.strip_prefix("http://"))
                .unwrap_or(repo_url),
        );

    // 分割路径，格式应该是: domain.com/owner/repo
    let parts: Vec<&str> = url.split('/').collect();
    if parts.len() < 3 {
        return Err(format!("无效的仓库URL格式: {repo_url}").into());
    }

    let owner = parts[1];
    let repo = parts[2];

    Ok((owner, repo))
}

/// 转换 GitHub releases 为统一格式
fn convert_github_releases(releases: Vec<GitHubRelease>) -> Vec<UnifiedRelease> {
    releases
        .into_iter()
        .map(|r| {
            let loader_asset_url = r
                .assets
                .iter()
                .find(|a| a.name == "pie-boot-loader-aarch64.bin")
                .map(|a| a.browser_download_url.clone());

            UnifiedRelease {
                tag_name: r.tag_name,
                draft: r.draft,
                prerelease: r.prerelease,
                loader_asset_url,
            }
        })
        .collect()
}

/// 转换 Gitee releases 为统一格式
fn convert_gitee_releases(releases: Vec<GiteeRelease>) -> Vec<UnifiedRelease> {
    releases
        .into_iter()
        .map(|r| {
            let loader_asset_url = r
                .assets
                .iter()
                .find(|a| a.name == "pie-boot-loader-aarch64.bin")
                .map(|a| a.browser_download_url.clone());

            UnifiedRelease {
                tag_name: r.tag_name,
                draft: r.draft,
                prerelease: r.prerelease,
                loader_asset_url,
            }
        })
        .collect()
}

/// 寻找最新的稳定版本
fn find_latest_stable_release(
    releases: Vec<UnifiedRelease>,
) -> Result<UnifiedRelease, Box<dyn std::error::Error + Send + Sync>> {
    let mut stable_releases: Vec<_> = releases
        .into_iter()
        .filter(|r| !r.draft && !r.prerelease)
        .filter(|r| r.loader_asset_url.is_some())
        .collect();

    if stable_releases.is_empty() {
        return Err("没有找到包含 pie-boot-loader-aarch64.bin 的稳定版本".into());
    }

    // 按版本号排序（简单字符串比较，可能需要更复杂的版本比较）
    stable_releases.sort_by(|a, b| version_compare(&b.tag_name, &a.tag_name));

    Ok(stable_releases.into_iter().next().unwrap())
}

/// 简单的版本比较
fn version_compare(a: &str, b: &str) -> std::cmp::Ordering {
    // 移除 'v' 前缀
    let a = a.strip_prefix('v').unwrap_or(a);
    let b = b.strip_prefix('v').unwrap_or(b);

    // 分割版本号
    let a_parts: Vec<u32> = a.split('.').filter_map(|s| s.parse().ok()).collect();
    let b_parts: Vec<u32> = b.split('.').filter_map(|s| s.parse().ok()).collect();

    // 逐个比较版本号
    for i in 0..std::cmp::max(a_parts.len(), b_parts.len()) {
        let a_part = a_parts.get(i).unwrap_or(&0);
        let b_part = b_parts.get(i).unwrap_or(&0);

        match a_part.cmp(b_part) {
            std::cmp::Ordering::Equal => continue,
            other => return other,
        }
    }

    std::cmp::Ordering::Equal
}

/// 下载 loader 二进制文件
async fn download_loader_bin(
    client: &reqwest::Client,
    release: &UnifiedRelease,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let download_url = release
        .loader_asset_url
        .as_ref()
        .ok_or("没有找到 pie-boot-loader-aarch64.bin 文件")?;

    println!("正在下载: pie-boot-loader-aarch64.bin");

    let response = client.get(download_url).send().await?;

    if !response.status().is_success() {
        return Err(format!("下载失败: {}", response.status()).into());
    }

    let bytes = response.bytes().await?;

    // 保存到目标位置
    let loader_dst = out_dir().join("loader.bin");
    let _ = std::fs::remove_file(&loader_dst);
    std::fs::write(&loader_dst, &bytes)?;

    println!("成功下载 {} 字节到 {}", bytes.len(), loader_dst.display());

    Ok(())
}
