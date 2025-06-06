// This is free and unencumbered software released into the public domain.

use serde::Deserialize;
use std::{error::Error, process::Command};

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

pub async fn try_install_from_github(
    module_name: &str,
    verbosity: u8,
) -> Result<bool, Box<dyn Error>> {
    let platform = detect_platform();

    // Fetch the latest release
    let release = match fetch_latest_release(module_name).await {
        Ok(release) => release,
        Err(_) => return Ok(false), // No release found
    };

    // Find a matching asset for our platform
    let asset = match find_matching_asset(&release.assets, module_name, &platform) {
        Some(asset) => asset,
        None => return Ok(false), // No matching asset
    };

    // Download and install the binary
    download_and_install_binary(asset, module_name, verbosity).await?;

    Ok(true)
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

async fn fetch_latest_release(module_name: &str) -> Result<GitHubRelease, Box<dyn Error>> {
    let url = format!(
        "https://api.github.com/repos/asimov-platform/asimov-{}-module/releases/latest",
        module_name
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
    // Look for pattern: asimov-{module}-{os}-{arch}[-{libc}].tar.gz
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

async fn download_and_install_binary(
    asset: &GitHubAsset,
    module_name: &str,
    _verbosity: u8,
) -> Result<(), Box<dyn Error>> {
    let client = super::http::http_client();
    let response = client.get(&asset.browser_download_url).send().await?;

    if !response.status().is_success() {
        return Err(format!("Failed to download asset: {}", response.status()).into());
    }

    let bytes = response.bytes().await?;

    // Create a temporary file
    let temp_path = std::env::temp_dir().join(&asset.name);
    std::fs::write(&temp_path, &bytes)?;

    // Extract the tar.gz file
    let output = Command::new("tar")
        .args(["-xzf", temp_path.to_str().unwrap()])
        .current_dir(std::env::temp_dir())
        .output()?;

    if !output.status.success() {
        return Err("Failed to extract tar.gz file".into());
    }

    let binary_name = format!("asimov-{}-module", module_name);
    let extracted_binary = std::env::temp_dir().join(&binary_name);

    if !extracted_binary.exists() {
        return Err(format!("Binary {} not found in extracted archive", binary_name).into());
    }

    // Install to ~/.cargo/bin or system path
    let home_dir = std::env::home_dir().ok_or("Could not find home directory")?;
    let install_dir = home_dir.join(".cargo").join("bin");

    std::fs::create_dir_all(&install_dir)?;

    let target_path = install_dir.join(&binary_name);
    std::fs::copy(&extracted_binary, &target_path)?;

    // Make executable on Unix systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&target_path)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&target_path, perms)?;
    }

    // Clean up temporary files
    let _ = std::fs::remove_file(&temp_path);
    let _ = std::fs::remove_file(&extracted_binary);

    Ok(())
}
