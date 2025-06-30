// This is free and unencumbered software released into the public domain.

use crate::{
    StandardOptions, SysexitsError,
    registry::{self, ModuleMetadata},
};
use asimov_env::{
    env::Env,
    envs::{CargoEnv, PythonEnv, RubyEnv},
    paths::asimov_root,
};
use color_print::{ceprintln, cprintln};
use serde_yml::Value;
use std::io::ErrorKind;

#[tokio::main]
pub async fn uninstall(
    module_names: Vec<String>,
    flags: &StandardOptions,
) -> Result<(), SysexitsError> {
    let remove_manifest = async |module_name: &str, manifest_file: &std::path::PathBuf| {
        match tokio::fs::remove_file(manifest_file).await {
            Ok(_) => {
                if flags.verbose > 1 {
                    cprintln!("<s,g>✓</> Removed manifest for module `{}`.", module_name);
                }
            }
            Err(err) if err.kind() == ErrorKind::NotFound => (),
            Err(err) => {
                tracing::error!(
                    "failed to remove manifest for module `{}`: {}",
                    module_name,
                    err
                );
                return Err(SysexitsError::from(err));
            }
        };
        Ok(())
    };

    for module_name in &module_names {
        let manifest_file = asimov_root()
            .join("modules")
            .join(format!("{module_name}.yaml"));

        let Ok(manifest) = tokio::fs::read(&manifest_file).await.inspect_err(|e| {
            tracing::warn!("unable to read manifest for module `{module_name}`: {e}")
        }) else {
            continue;
        };
        let Ok(manifest) = serde_yml::from_slice::<Value>(&manifest)
            .inspect_err(|e| tracing::error!("malformed manifest for module `{module_name}`: {e}"))
        else {
            remove_manifest(module_name, &manifest_file).await?;
            continue;
        };

        let binaries = manifest["provides"]["flows"]
            .as_sequence()
            .into_iter()
            .flatten()
            .flat_map(Value::as_str);

        for binary in binaries {
            let path = asimov_root().join("libexec").join(binary);
            match tokio::fs::remove_file(&path).await {
                Ok(_) => {
                    if flags.verbose > 1 {
                        cprintln!("<s,g>✓</> Removed binary `{}`.", path.display());
                    }
                }
                Err(err) if err.kind() == ErrorKind::NotFound => (),
                Err(err) => {
                    tracing::error!("failed to remove binary `{}`: {err}", path.display());
                    return Err(SysexitsError::from(err));
                }
            }
        }

        remove_manifest(module_name, &manifest_file).await?;
    }

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
                tracing::debug!("skipping registry uninstall for unknown module: {module_name}");
                continue;
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
                tracing::error!(
                    "failed to uninstall module `{}`: missing {} environment",
                    module.name,
                    module.r#type.to_string()
                );
                return Err(SysexitsError::EX_UNAVAILABLE);
            }
            Err(error) => {
                tracing::error!("failed to uninstall module `{}`: {}", module.name, error);
                return Err(SysexitsError::EX_OSERR);
            }
            Ok(status) if !status.success() => {
                tracing::error!(
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
