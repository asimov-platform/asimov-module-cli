// This is free and unencumbered software released into the public domain.

use asimov_env::paths::asimov_root;
use clientele::{SysexitsError, SysexitsError::*};
use color_print::{ceprintln, cprintln};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{error::Error, fs::Permissions, path::Path};
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};

#[derive(Debug)]
struct PlatformInfo {
    os: String,
    arch: String,
    libc: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    name: String,
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

pub async fn fetch_latest_release(module_name: &str) -> Result<String, SysexitsError> {
    let url = format!(
        "https://api.github.com/repos/asimov-modules/asimov-{}-module/releases/latest",
        module_name
    );
    let client = crate::registry::http::http_client();

    let response = client.get(url).send().await.map_err(|e| {
        ceprintln!("<s,r>error:</> request failed: {}", e);
        EX_UNAVAILABLE
    })?;

    if response.status() != 200 {
        ceprintln!(
            "<s,r>error:</> request failed: HTTP status {}",
            response.status()
        );
        return Err(EX_UNAVAILABLE);
    }

    response
        .json::<GitHubRelease>()
        .await
        .map_err(|e| {
            ceprintln!("<s,r>error:</> failed to read the response: {}", e);
            EX_UNAVAILABLE
        })
        .map(|release| release.name)
}

pub async fn install_from_github(
    module_name: &str,
    version: &str,
    verbosity: u8,
) -> Result<(), SysexitsError> {
    let platform = detect_platform();
    if verbosity > 1 {
        cprintln!("<s,c>»</> Searching for assets on GitHub...");
    }
    let release = fetch_release(module_name, version).await?;
    let Some(asset) = find_matching_asset(&release.assets, module_name, &platform) else {
        ceprintln!(
            "<s,r>error:</> no matching asset found for platform {}-{}",
            platform.os,
            platform.arch
        );
        return Err(EX_UNAVAILABLE);
    };

    let temp_dir = tempfile::Builder::new()
        .prefix("asimov-module-cli-")
        .tempdir()?;

    if verbosity > 1 {
        cprintln!("<s,c>»</> Downloading asset from GitHub...");
    }
    let download = download_asset(asset, temp_dir.path()).await.map_err(|e| {
        ceprintln!("<s,r>error:</> failed to download asset: {}", e);
        EX_UNAVAILABLE
    })?;
    if verbosity > 0 {
        cprintln!("<s,g>✓</> Downloaded asset `{}`", asset.name);
    }

    match fetch_checksum(asset).await {
        Ok(None) => {
            if verbosity > 1 {
                cprintln!("<s,y>warning:</> No checksum file found, skipping verification");
            }
        }
        Ok(Some(checksum)) => {
            if verbosity > 1 {
                cprintln!("<s,c>»</> Verifying checksum...");
            }
            verify_checksum(&download, &checksum).await?;
            if verbosity > 0 {
                cprintln!("<s,g>✓</> Verified checksum");
            }
        }
        Err(err) => {
            ceprintln!("<s,r>error:</> error while fetching checksum file: {}", err);
            return Err(EX_UNAVAILABLE);
        }
    }

    if verbosity > 1 {
        cprintln!("<s,c>»</> Installing binaries...");
    }
    install_binaries(&download, verbosity).await.map_err(|e| {
        ceprintln!("<s,r>error:</> failed to install binaries: {}", e);
        EX_UNAVAILABLE
    })?;

    Ok(())
}

pub async fn install_module_manifest(
    module_name: &str,
    version: &str,
) -> Result<(), SysexitsError> {
    let module_dir = asimov_root().join("modules");
    tokio::fs::create_dir_all(&module_dir).await?;

    let url = format!(
        "https://raw.githubusercontent.com/asimov-modules/asimov-{}-module/{}/.asimov/module.yaml",
        module_name, version
    );

    let response = crate::registry::http::http_client()
        .get(url)
        .send()
        .await
        .map_err(|_| EX_UNAVAILABLE)?;

    if response.status() != 200 {
        ceprintln!(
            "<s,r>error:</> failed to fetch module manifest: HTTP status {}",
            response.status()
        );
        return Err(EX_UNAVAILABLE);
    }

    let manifest = response.bytes().await.map_err(|_| EX_UNAVAILABLE)?;

    let manifest_filename = module_dir.join(format!("{}.yaml", module_name));
    let mut manifest_file = tokio::fs::File::create(manifest_filename).await?;

    use tokio::io::AsyncWriteExt as _;
    manifest_file.write_all(&manifest).await?;

    Ok(())
}

fn detect_platform() -> PlatformInfo {
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    let os = "unknown";
    #[cfg(target_os = "macos")]
    let os = "macos";
    #[cfg(target_os = "linux")]
    let os = "linux";
    #[cfg(target_os = "windows")]
    let os = "windows";

    #[cfg(not(any(target_arch = "aarch64", target_arch = "arm", target_arch = "x86_64")))]
    let arch = "unknown";
    #[cfg(any(target_arch = "aarch64", target_arch = "arm"))]
    let arch = "arm";
    #[cfg(all(target_arch = "x86_64", not(target_os = "windows")))]
    let arch = "x86";
    #[cfg(all(target_arch = "x86_64", target_os = "windows"))]
    let arch = "x64";

    #[cfg(not(any(target_env = "musl", target_env = "gnu")))]
    let libc = None;
    #[cfg(target_env = "musl")]
    let libc = Some("musl".to_string());
    #[cfg(target_env = "gnu")]
    let libc = Some("gnu".to_string());

    PlatformInfo {
        os: os.into(),
        arch: arch.into(),
        libc,
    }
}

async fn fetch_release(module_name: &str, version: &str) -> Result<GitHubRelease, Box<dyn Error>> {
    let url = format!(
        "https://api.github.com/repos/asimov-modules/asimov-{}-module/releases/tags/{}",
        module_name, version
    );

    let client = super::http::http_client();
    let response = client.get(&url).send().await?;

    if !response.status().is_success() {
        return Err(format!("GitHub API request failed: {}", response.status()).into());
    }

    let release: GitHubRelease = response.json().await?;
    Ok(release)
}

fn find_matching_asset<'a>(
    assets: &'a [GitHubAsset],
    module_name: &str,
    platform: &PlatformInfo,
) -> Option<&'a GitHubAsset> {
    let patterns = if let Some(libc) = &platform.libc {
        vec![
            format!(
                "asimov-{}-module-{}-{}-{}.tar.gz",
                module_name, platform.os, platform.arch, libc
            ),
            format!(
                "asimov-{}-module-{}-{}-{}.zip",
                module_name, platform.os, platform.arch, libc
            ),
            format!(
                "asimov-{}-module-{}-{}.tar.gz",
                module_name, platform.os, platform.arch
            ),
            format!(
                "asimov-{}-module-{}-{}.zip",
                module_name, platform.os, platform.arch
            ),
        ]
    } else {
        vec![
            format!(
                "asimov-{}-module-{}-{}.tar.gz",
                module_name, platform.os, platform.arch
            ),
            format!(
                "asimov-{}-module-{}-{}.zip",
                module_name, platform.os, platform.arch
            ),
        ]
    };

    for pattern in patterns {
        if let Some(asset) = assets.iter().find(|asset| asset.name == pattern) {
            return Some(asset);
        }
    }

    None
}

async fn fetch_checksum(asset: &GitHubAsset) -> Result<Option<String>, Box<dyn Error>> {
    let checksum_url = format!("{}.sha256", asset.browser_download_url);

    let client = super::http::http_client();

    let response = client.get(&checksum_url).send().await?;

    if response.status() == 404 {
        return Ok(None);
    }

    if !response.status().is_success() {
        return Err(format!("Failed to checksum asset: {}", response.status()).into());
    }

    Ok(Some(response.text().await?.trim().to_string()))
}

async fn download_asset(
    asset: &GitHubAsset,
    dst_dir: &Path,
) -> Result<std::path::PathBuf, Box<dyn Error>> {
    let client = super::http::http_client();
    let mut response = client.get(&asset.browser_download_url).send().await?;

    if !response.status().is_success() {
        return Err(format!("Failed to download asset: {}", response.status()).into());
    }

    let asset_path = dst_dir.join(&asset.name);
    let mut dst = tokio::fs::File::create(&asset_path)
        .await
        .map_err(|e| format!("Failed to create file for download: {}", e))?;
    while let Some(chunk) = response.chunk().await? {
        dst.write_all(&chunk).await?;
    }
    dst.flush().await?;

    Ok(asset_path)
}

async fn verify_checksum(
    binary_path: &Path,
    expected_checksum: &str,
) -> Result<(), Box<dyn Error>> {
    let mut hasher = Sha256::new();
    let mut file = tokio::fs::File::open(binary_path).await?;
    let mut buf = vec![0u8; 10 * 1024];
    loop {
        let n = file.read(&mut buf).await?;
        if n == 0 {
            break; // End of file
        }
        hasher.update(&buf[..n]);
    }
    let actual_checksum = format!("{:x}", hasher.finalize());

    // Extract just the hash part from expected (in case it has filename)
    let expected_hash = expected_checksum
        .split_whitespace()
        .next()
        .unwrap_or(expected_checksum);

    if actual_checksum != expected_hash {
        return Err(format!(
            "Checksum verification failed: expected {}, got {}",
            expected_hash, actual_checksum
        )
        .into());
    }

    Ok(())
}

async fn install_binaries(src_asset: &Path, verbosity: u8) -> Result<(), Box<dyn Error>> {
    let install_dir = asimov_root().join("libexec");
    tokio::fs::create_dir_all(&install_dir).await?;

    let temp_extract_dir = src_asset
        .parent()
        .expect("Incorrect asset directory")
        .join("extracted");
    tokio::fs::create_dir_all(&temp_extract_dir).await?;

    tokio::task::spawn_blocking({
        let src_asset = src_asset.to_owned();
        let src_name = src_asset.to_string_lossy().into_owned();
        let dst = temp_extract_dir.clone();
        use std::io::{Error, Result};
        move || -> Result<()> {
            let asset_file = std::fs::File::open(&src_asset)?;
            if src_name.ends_with(".tar.gz") {
                let gz = flate2::read::GzDecoder::new(asset_file);
                let mut archive = tar::Archive::new(gz);
                archive.unpack(&dst)?;
            } else if src_name.ends_with(".zip") {
                let mut archive = zip::ZipArchive::new(asset_file)?;
                archive.extract(&dst)?;
            } else {
                return Err(Error::other("Unsupported format"));
            }
            Ok(())
        }
    })
    .await??;

    let mut read_dir = tokio::fs::read_dir(&temp_extract_dir).await?;

    while let Some(entry) = read_dir.next_entry().await? {
        if !entry.file_type().await?.is_file() {
            continue;
        }
        let name = entry.file_name();
        let mut src = tokio::fs::File::open(entry.path()).await?;
        let dst_path = install_dir.join(&name);
        let mut dst = tokio::fs::File::create(&dst_path).await?;
        tokio::io::copy(&mut src, &mut dst).await?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            tokio::fs::set_permissions(dst_path, Permissions::from_mode(0o755)).await?;
        }

        if verbosity > 0 {
            cprintln!("<s,g>✓</> Installed binary `{}`", name.to_string_lossy());
        }
    }

    Ok(())
}
