// This is free and unencumbered software released into the public domain.

use crate::{
    registry::{self, ModuleMetadata},
    StandardOptions, SysexitsError,
};
use asimov_env::{
    env::Env,
    envs::{CargoEnv, PythonEnv, RubyEnv},
};
use color_print::{ceprintln, cprintln};
use std::io::ErrorKind;

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
                if module.is_installed()? {
                    modules_to_uninstall.push(module.clone());
                }
            }
            None => {
                ceprintln!("<s><r>error:</></> unknown module: {}", module_name);
                return Err(SysexitsError::EX_UNAVAILABLE);
            }
        }
    }

    let venv_verbosity = if flags.debug { flags.verbose + 1 } else { 0 };

    for module in modules_to_uninstall {
        use registry::ModuleType::*;

        if flags.verbose > 1 {
            cprintln!("<s><c>»</></> Uninstalling the module `{}`...", module.name,);
        }

        let result = match module.r#type {
            Rust => CargoEnv::default().uninstall_module(&module.name, Some(venv_verbosity)),
            Ruby => RubyEnv::default().uninstall_module(&module.name, Some(venv_verbosity)),
            Python => PythonEnv::default().uninstall_module(&module.name, Some(venv_verbosity)),
        };

        match result {
            Err(error) if error.kind() == ErrorKind::NotFound => {
                ceprintln!(
                    "<s><r>error:</></> failed to uninstall module `{}`: missing {} environment",
                    module.name,
                    module.r#type.to_string()
                );
                return Err(SysexitsError::EX_UNAVAILABLE);
            }
            Err(error) => {
                ceprintln!(
                    "<s><r>error:</></> failed to uninstall module `{}`: {}",
                    module.name,
                    error
                );
                return Err(SysexitsError::EX_OSERR);
            }
            Ok(status) if !status.success() => {
                ceprintln!(
                    "<s><r>error:</></> failed to uninstall module `{}`: exit code {}",
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
