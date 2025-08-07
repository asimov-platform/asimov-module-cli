// This is free and unencumbered software released into the public domain.

use crate::{
    StandardOptions,
    SysexitsError::{self, *},
};
use asimov_module::{ConfigurationVariable, InstalledModuleManifest, ReadVarError};
use color_print::{ceprintln, cprintln};

#[tokio::main]
pub async fn install(
    module_names: Vec<String>,
    flags: &StandardOptions,
) -> Result<(), SysexitsError> {
    let registry = asimov_registry::Registry::default();
    let installer = asimov_installer::Installer::default();
    for module_name in module_names {
        if !registry
            .is_module_installed(&module_name)
            .await
            .unwrap_or(false)
        {
            let latest = installer
                .fetch_latest_release(&module_name)
                .await
                .map_err(|e| {
                    tracing::error!(
                        "unable to find latest release for module `{module_name}`: {e}"
                    );
                    EX_UNAVAILABLE
                })?;

            if flags.verbose > 0 {
                cprintln!(
                    "<s,g>✓</> Found latest version <s>{latest}</> for module `<s>{module_name}</>`."
                );
            }

            if flags.verbose > 1 {
                cprintln!("<s,c>»</> Installing module `<s>{module_name}</>`...");
            }

            installer
                .install_module(&module_name, &latest)
                .await
                .map_err(|e| {
                    tracing::error!("failed to install for module `{module_name}`: {e}");
                    EX_UNAVAILABLE
                })?;

            if flags.verbose > 0 {
                cprintln!("<s,g>✓</> Installed module `<s>{module_name}</>`.");
            }
        } else {
            if flags.verbose > 0 {
                cprintln!("<s,g>✓</> Module `<s>{module_name}</>` is already installed.");
            }
        }

        if registry
            .is_module_enabled(&module_name)
            .await
            .unwrap_or(false)
        {
            continue;
        }

        let manifest = registry.read_manifest(&module_name).await.map_err(|e| {
            tracing::error!("failed to read module manifest for `{module_name}`: {e}");
            EX_UNAVAILABLE
        })?;

        let variables = manifest
            .manifest
            .config
            .iter()
            .flat_map(|conf| conf.variables.iter());

        let mut missing_variables = Vec::new();
        for var in variables {
            if var.default_value.is_some() {
                continue;
            }
            match manifest.manifest.variable(&var.name, None) {
                Ok(_) => (),
                Err(ReadVarError::UnconfiguredVar(_)) => {
                    missing_variables.push(var);
                },
                Err(e) => {
                    tracing::error!(
                        "failed to read configuration variable `{}` for module `{module_name}`: {e}",
                        var.name
                    );
                    return Err(EX_UNAVAILABLE);
                },
            }
        }

        if missing_variables.is_empty() {
            registry.enable_module(&module_name).await.map_err(|e| {
                tracing::error!("failed to enable installed module `{module_name}`: {e}");
                EX_UNAVAILABLE
            })?;
        } else {
            ceprintln!(
                "<s,y>warn:</> Module `<s>{module_name}</>` can't be enabled automatically due to missing configuration."
            );
            ceprintln!("<s,dim>hint:</> Module `<s>{module_name}</>` requires configuration:");

            for var in missing_variables {
                let desc_suffix = if let Some(ref desc) = var.description {
                    format!(" (Description: \"{desc}\")")
                } else {
                    String::new()
                };

                ceprintln!(
                    "<s,dim>hint:</>   Missing variable: <s>{}</s>{}",
                    var.name,
                    desc_suffix
                );

                if let Some(ref env) = var.environment {
                    ceprintln!(
                        "<s,dim>hint:</>   Alternative: set environment variable: <s>{env}</>"
                    );
                }
            }

            ceprintln!("<s,dim>hint:</>   To configure: <s>asimov module config {module_name}</s>");
            ceprintln!("<s,dim>hint:</>   To enable: <s>asimov module enable {module_name}</s>");
        }
    }

    Ok(())
}
