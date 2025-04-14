// This is free and unencumbered software released into the public domain.

use crate::{
    registry::{self, ModuleMetadata},
    StandardOptions, SysexitsError,
};
use asimov_env::{
    paths::{python_env, ruby_env},
    tools::{cargo, python, ruby},
};
use std::{io::ErrorKind, process::Command};

#[tokio::main]
pub async fn install(
    module_names: Vec<String>,
    _flags: &StandardOptions,
) -> Result<(), SysexitsError> {
    let mut modules_to_install: Vec<ModuleMetadata> = vec![];

    for module_name in module_names {
        if registry::is_installed(&module_name) {
            continue; // skip already installed modules
        }

        match registry::fetch_module(&module_name).await {
            Some(module) => {
                modules_to_install.push(module.clone());
            }
            None => {
                eprintln!("unknown module: {}", module_name);
                return Err(SysexitsError::EX_UNAVAILABLE);
            }
        }
    }

    for module in modules_to_install {
        use registry::ModuleType::*;
        let package_name = format!("asimov-{}-module", module.name);

        let result = match module.r#type {
            Rust => Command::new(cargo().unwrap().as_ref())
                .args(["install", &package_name])
                .status(),
            Ruby => {
                let rbenv = ruby_env();
                if !rbenv.is_dir() {
                    // Create the directory if it doesn't exist:
                    std::fs::create_dir_all(&rbenv)?;
                }
                Command::new(ruby().unwrap().as_ref())
                    .args(["-S", "gem"])
                    .args([
                        "install",
                        "--install-dir",
                        &rbenv.to_string_lossy(),
                        "--prerelease",
                        "--no-document",
                        &package_name,
                    ])
                    .status()
            }
            Python => {
                let venv = python_env();
                if !venv.is_dir() {
                    // Create the directory if it doesn't exist:
                    std::fs::create_dir_all(&venv)?;

                    // Create the virtual environment if it doesn't exist:
                    Command::new(python().unwrap().as_ref())
                        .args(["-m", "venv", &venv.to_string_lossy()])
                        .status()?;
                }
                Command::new(venv.join("bin/python3"))
                    .args(["-m", "pip"])
                    .args(["install", &package_name])
                    .env("VIRTUAL_ENV", venv.as_os_str())
                    .status()
            }
        };

        match result {
            Err(error) if error.kind() == ErrorKind::NotFound => {
                eprintln!(
                    "failed to install module `{}`: missing {} environment",
                    module.name,
                    module.r#type.to_string()
                );
                return Err(SysexitsError::EX_UNAVAILABLE);
            }
            Err(error) => {
                eprintln!("failed to install module `{}`: {}", module.name, error);
                return Err(SysexitsError::EX_OSERR);
            }
            Ok(status) if !status.success() => {
                eprintln!(
                    "failed to install module `{}`: exit code {}",
                    module.name,
                    status.code().unwrap_or_default()
                );
                return Err(SysexitsError::EX_SOFTWARE);
            }
            Ok(_) => {}
        }
    }

    Ok(())
}
