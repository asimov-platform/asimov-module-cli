// This is free and unencumbered software released into the public domain.

use crate::{
    StandardOptions,
    SysexitsError::{self, *},
};
use color_print::cprintln;

#[tokio::main]
pub async fn upgrade(
    module_names: Vec<String>,
    flags: &StandardOptions,
) -> Result<(), SysexitsError> {
    let registry = asimov_registry::Registry::default();
    let installer = asimov_installer::Installer::default();

    let module_names = if !module_names.is_empty() {
        module_names
    } else {
        registry
            .installed_modules()
            .await
            .map_err(|e| {
                tracing::error!("failed to read installed modules: {e}");
                EX_UNAVAILABLE
            })?
            .into_iter()
            .map(|manifest| manifest.manifest.name)
            .collect()
    };

    for module_name in module_names {
        let current = registry.module_version(&module_name).await.map_err(|e| {
            tracing::error!("failed to read installed version of `{module_name}`");
            EX_UNAVAILABLE
        })?;

        let latest = installer
            .fetch_latest_release(&module_name)
            .await
            .map_err(|e| {
                tracing::error!("unable to find latest release for module `{module_name}`: {e}");
                EX_UNAVAILABLE
            })?;

        if current.is_some_and(|current| current == latest) {
            if flags.verbose > 0 {
                cprintln!(
                    "<s,g>✓</> Module `{module_name}` already has latest version <s>{latest}</> installed."
                );
            }
            continue;
        }

        if flags.verbose > 1 {
            cprintln!("<s,c>»</> Upgrading module `{module_name}`...");
        }

        installer
            .upgrade_module(&module_name, &latest)
            .await
            .map_err(|e| {
                tracing::error!("module upgrade failed for `{module_name}`: {e}");
                EX_UNAVAILABLE
            })?;

        if flags.verbose > 0 {
            cprintln!("<s,g>✓</> Upgraded module `{module_name}` to version <s>{latest}</>.");
        }
    }
    Ok(())
}
