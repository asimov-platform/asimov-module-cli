// This is free and unencumbered software released into the public domain.

use crate::{
    StandardOptions,
    SysexitsError::{self, *},
    registry::{self, ModuleMetadata},
};
use asimov_env::{
    env::Env,
    envs::{CargoEnv, PythonEnv, RubyEnv},
    paths::asimov_root,
};
use color_print::{ceprintln, cprintln};
use std::io::ErrorKind;

#[tokio::main]
pub async fn install(
    module_names: Vec<String>,
    flags: &StandardOptions,
) -> Result<(), SysexitsError> {
    let mut modules_to_install: Vec<ModuleMetadata> = vec![];

    let venv_verbosity = if flags.debug { flags.verbose + 1 } else { 0 };

    for module_name in module_names {
        let release = registry::github::fetch_latest_release(&module_name)
            .await
            .inspect_err(|e| {
                tracing::error!("unable to find latest release for module `{module_name}`: {e}")
            })?;

        if flags.verbose > 0 {
            cprintln!(
                "<s,c>✓</> Found latest version `{}` for module `{}`.",
                release,
                module_name,
            );
        }

        registry::github::install_module_manifest(&module_name, &release)
            .await
            .inspect_err(|e| {
                tracing::error!(
                    "failed to install module manifest for `{module_name}`: exit code {e}"
                )
            })?;
        if flags.verbose > 1 {
            cprintln!(
                "<s,c>✓</> Fetched module manifest for module `{}`",
                module_name,
            );
        }

        let github_install_result =
            registry::github::install_from_github(&module_name, &release, flags.verbose).await;

        match github_install_result {
            Ok(_) => {
                if flags.verbose > 0 {
                    cprintln!(
                        "<s,g>✓</> Installed the module `{module_name}` from GitHub releases."
                    );
                }
            }
            Err(err) => {
                tracing::warn!(
                    "Install from GitHub releases failed: {err}, trying install from registry..."
                );

                let module = match registry::fetch_module(&module_name).await {
                    Some(module) => {
                        if !module.is_installed()? {
                            module
                        } else {
                            continue;
                        }
                    }
                    None => {
                        tracing::error!("unknown module: {module_name}");
                        return Err(EX_UNAVAILABLE);
                    }
                };

                use registry::ModuleType::*;

                if flags.verbose > 1 {
                    cprintln!(
                        "<s,c>»</> Installing the module `{}` from {}...",
                        module.name,
                        module.r#type.origin(),
                    );
                }

                let result = match module.r#type {
                    Rust => CargoEnv::default().install_module(&module.name, Some(venv_verbosity)),
                    Ruby => RubyEnv::default().install_module(&module.name, Some(venv_verbosity)),
                    Python => {
                        PythonEnv::default().install_module(&module.name, Some(venv_verbosity))
                    }
                };

                match result {
                    Err(error) if error.kind() == ErrorKind::NotFound => {
                        tracing::error!(
                            "failed to install module `{}`: missing {} environment",
                            module.name,
                            module.r#type.to_string()
                        );
                        return Err(EX_UNAVAILABLE);
                    }
                    Err(err) => {
                        tracing::error!("failed to install module `{}`: {err}", module.name);
                        return Err(EX_OSERR);
                    }
                    Ok(status) if !status.success() => {
                        tracing::error!(
                            "failed to install module `{}`: exit code {}",
                            module.name,
                            status.code().unwrap_or_default()
                        );
                        return Err(EX_SOFTWARE);
                    }
                    Ok(_) => {
                        if flags.verbose > 0 {
                            cprintln!(
                                "<s,g>✓</> Installed the module `{}` from {}.",
                                module.name,
                                module.r#type.origin(),
                            );
                        }
                    }
                };
            }
        }
    }

    Ok(())
}
