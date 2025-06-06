// This is free and unencumbered software released into the public domain.

use color_print::cprintln;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    error::Error,
    fs::Permissions,
    path::Path,
    process::{Command, ExitStatus},
};
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};

use super::ModuleMetadata;

#[derive(Debug)]
struct PlatformInfo {
    os: String,
    arch: String,
    libc: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

pub async fn install_from_github(
    module: &ModuleMetadata,
    verbosity: u8,
) -> Result<ExitStatus, Box<dyn Error>> {
    let platform = detect_platform();
    let release = fetch_release(module).await?;
    let asset = find_matching_asset(&release.assets, &module.name, &platform).ok_or_else(|| {
        format!(
            "No matching asset found for platform {}-{}",
            platform.os, platform.arch
        )
    })?;

    let temp_dir = tempfile::Builder::new()
        .prefix("asimov-module-cli-")
        .disable_cleanup(true)
        .tempdir()?;

    if verbosity > 1 {
        cprintln!("<s,c>»</> Downloading asset from github...");
    }
    let download = download_asset(asset, temp_dir.path())
        .await
        .map_err(|e| format!("Failed to download asset: {}", e))?;
    if verbosity > 0 {
        cprintln!("<s><g>✓</></> Downloaded asset `{}`", asset.name);
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
                cprintln!("<s><g>✓</></> Verified checksum");
            }
        }
        Err(err) => {
            return Err(format!("Error while fetching checksum file: {}", err).into());
        }
    }

    if verbosity > 1 {
        cprintln!("<s,c>»</> Installing binaries...");
    }
    install_binaries(&download, verbosity)
        .await
        .map_err(|e| format!("Failed to install binaries: {}", e))?;

    Ok(ExitStatus::default())
}

fn detect_platform() -> PlatformInfo {
    let os = match std::env::consts::OS {
        "linux" => "linux",
        "macos" => "macos",
        "windows" => "windows",
        _ => "unknown",
    }
    .to_string();

    let arch = match std::env::consts::ARCH {
        "x86_64" => "x86",
        "aarch64" => "arm",
        "arm" => "arm",
        _ => "unknown",
    }
    .to_string();

    // For Linux, try to detect libc type
    let libc = if os == "linux" {
        // Simple heuristic: check if we're likely using musl
        if std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default() == "musl" {
            Some("musl".to_string())
        } else {
            Some("gnu".to_string())
        }
    } else {
        None
    };

    PlatformInfo { os, arch, libc }
}

async fn fetch_release(module: &ModuleMetadata) -> Result<GitHubRelease, Box<dyn Error>> {
    let url = format!(
        "https://api.github.com/repos/asimov-modules/asimov-{}-module/releases/tags/{}",
        &module.name, &module.version
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
                "asimov-{}-module-{}-{}.tar.gz",
                module_name, platform.os, platform.arch
            ),
        ]
    } else {
        vec![format!(
            "asimov-{}-module-{}-{}.tar.gz",
            module_name, platform.os, platform.arch
        )]
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
    drop(dst);

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
    let home_dir = std::env::home_dir().ok_or("Could not find home directory")?;
    let install_dir = home_dir.join(".cargo").join("bin");
    tokio::fs::create_dir_all(&install_dir).await?;

    let src_dir = src_asset.parent().ok_or("Invalid source path")?;

    let file = std::fs::File::open(src_asset)?;
    let reader = std::io::BufReader::new(file);

    if src_asset.to_string_lossy().ends_with(".tar.gz") {
        use flate2::read::GzDecoder;
        use tar::Archive;
        Archive::new(GzDecoder::new(reader))
            .entries()?
            .into_iter()
            .try_for_each(|f| {
                let mut f = f?;
                f.unpack_in(&install_dir).and_then(|_| {
                    // Make executable on Unix systems
                    let name = f.path()?.file_name().unwrap().to_owned();
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        std::fs::set_permissions(
                            &install_dir.join(&name),
                            Permissions::from_mode(0o755),
                        )?;
                    }
                    if verbosity > 0 {
                        cprintln!(
                            "<s><g>✓</></> Installed binary `{}`",
                            name.to_string_lossy()
                        );
                    }
                    Ok(())
                })
            })?;
    } else if src_asset.to_string_lossy().ends_with(".zip") {
        use zip::ZipArchive;

        let mut archive = ZipArchive::new(reader)?;
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = src_dir.join(file.name());

            if file.name().ends_with('/') {
                std::fs::create_dir_all(&outpath)?;
            } else {
                if let Some(parent) = outpath.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let mut out = std::fs::File::create(&outpath)?;
                std::io::copy(&mut file, &mut out)?;
            }
        }
    } else {
        return Err("Unsupported format".into());
    }

    // let mut read_dir = tokio::fs::read_dir(src_dir).await?;

    // while let Some(file) = read_dir.next_entry().await? {
    //     let path = file.path();
    //     let Some(name) = path.file_name() else {
    //         continue;
    //     };
    //     if path.to_string_lossy().ends_with(".tar.gz") {
    //         continue;
    //     }
    //     if path.to_string_lossy().ends_with(".zip") {
    //         continue;
    //     }
    //     let target_path = install_dir.join(name);
    //     tokio::fs::copy(&path, &target_path).await?;

    //     // Make executable on Unix systems
    //     #[cfg(unix)]
    //     {
    //         use std::os::unix::fs::PermissionsExt;
    //         tokio::fs::set_permissions(&target_path, Permissions::from_mode(0o755)).await?;
    //     }

    //     if verbosity > 0 {
    //         cprintln!(
    //             "<s><g>✓</></> Installed binary `{}`",
    //             name.to_string_lossy()
    //         );
    //     }
    // }

    Ok(())
}
