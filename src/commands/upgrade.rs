// This is free and unencumbered software released into the public domain.

use color_print::{ceprintln, cprintln};

use crate::{
    StandardOptions, SysexitsError,
    registry::github::{
        fetch_latest_release, install_from_github, installed_modules, installed_version,
    },
};

#[tokio::main]
pub async fn upgrade(
    module_names: Vec<String>,
    flags: &StandardOptions,
) -> Result<(), SysexitsError> {
    let module_names = if !module_names.is_empty() {
        module_names
    } else {
        installed_modules()
            .await
            .inspect_err(|e| ceprintln!("<s,r>error:</> failed to read installed modules: {e}"))?
    };
    for module_name in module_names {
        let Some(installed) = installed_version(&module_name).await? else {
            ceprintln!("<s,y>warn:</> Module '{module_name}' is not installed, skipping...");
            continue;
        };

        let latest = fetch_latest_release(&module_name).await.inspect_err(|_| {
            ceprintln!("<s,r>error:</> failed to check latest release version of '{module_name}'")
        })?;

        if installed == latest {
            if flags.verbose > 0 {
                cprintln!("<s,g>✓</> Module '{module_name}' already has latest version installed");
            }
            continue;
        }

        if flags.verbose > 1 {
            cprintln!("<s,c>»</>Upgrading module '{module_name}'...");
        }

        install_from_github(&module_name, &latest, flags.verbose)
            .await
            .inspect_err(|_| {
                ceprintln!("<s,r>error:</> failed to upgrade module '{module_name}'")
            })?;

        if flags.verbose > 0 {
            cprintln!("<s,g>✓</> Upgraded module '{module_name}' to version '{latest}'.");
        }
    }
    Ok(())
}
