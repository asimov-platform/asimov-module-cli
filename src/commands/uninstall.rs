// This is free and unencumbered software released into the public domain.

use crate::{
    registry::{self, ModuleMetadata},
    StandardOptions, SysexitsError,
};
use asimov_env::tools::{cargo, PythonEnv, RubyEnv};
use color_print::cprintln;
use std::{io::ErrorKind, process::Command};

#[tokio::main]
pub async fn uninstall(
    module_names: Vec<String>,
    flags: &StandardOptions,
) -> Result<(), SysexitsError> {
    let mut modules_to_uninstall: Vec<ModuleMetadata> = vec![];

    for module_name in module_names {
        // if !registry::is_installed(&module_name) {
        //     continue; // skip not installed modules
        // }

        match registry::fetch_module(&module_name).await {
            Some(module) => {
                modules_to_uninstall.push(module.clone());
            }
            None => {
                eprintln!("unknown module: {}", module_name);
                return Err(SysexitsError::EX_UNAVAILABLE);
            }
        }
    }

    let venv_verbosity = if flags.debug { flags.verbose + 1 } else { 0 };

    for module in modules_to_uninstall {
        use registry::ModuleType::*;
        let package_name = format!("asimov-{}-module", module.name);

        if flags.verbose > 1 {
            cprintln!("<s><c>»</></> Uninstalling the module `{}`...", module.name,);
        }

        let result = match module.r#type {
            Rust => Command::new(cargo().unwrap().as_ref())
                .args(["uninstall", &package_name])
                .status(),
            Ruby => {
                let rbenv = RubyEnv::default();
                if !rbenv.exists() {
                    rbenv.create()?;
                }
                rbenv
                    .gem_command("uninstall", venv_verbosity)
                    .args(["--all", "--executables", &package_name])
                    .status()
            }
            Python => {
                let venv = PythonEnv::default();
                if !venv.exists() {
                    venv.create()?;
                }
                venv.pip_command("uninstall", venv_verbosity)
                    .args(["--yes", &package_name])
                    .status()
            }
        };

        match result {
            Err(error) if error.kind() == ErrorKind::NotFound => {
                eprintln!(
                    "failed to uninstall module `{}`: missing {} environment",
                    module.name,
                    module.r#type.to_string()
                );
                return Err(SysexitsError::EX_UNAVAILABLE);
            }
            Err(error) => {
                eprintln!("failed to uninstall module `{}`: {}", module.name, error);
                return Err(SysexitsError::EX_OSERR);
            }
            Ok(status) if !status.success() => {
                eprintln!(
                    "failed to uninstall module `{}`: exit code {}",
                    module.name,
                    status.code().unwrap_or_default()
                );
                return Err(SysexitsError::EX_SOFTWARE);
            }
            Ok(_) => {
                if flags.verbose > 0 {
                    cprintln!("<s><g>✓</></> Uninstalled the module `{}`.", module.name);
                }
            }
        }
    }

    Ok(())
}
