// This is free and unencumbered software released into the public domain.

use crate::{
    StandardOptions,
    SysexitsError::{self, *},
};
use color_print::cprintln;

#[tokio::main]
pub async fn install(
    module_names: Vec<String>,
    flags: &StandardOptions,
) -> Result<(), SysexitsError> {
    let installer = asimov_installer::Installer::default();
    for module_name in module_names {
        let latest = installer
            .fetch_latest_release(&module_name)
            .await
            .map_err(|e| {
                tracing::error!("unable to find latest release for module `{module_name}`: {e}");
                EX_UNAVAILABLE
            })?;

        if flags.verbose > 0 {
            cprintln!("<s,g>✓</> Found latest version <s>{latest}</> for module `{module_name}`.");
        }

        if flags.verbose > 1 {
            cprintln!("<s,c>»</> Installing module `{module_name}`...");
        }

        installer
            .install_module(&module_name, &latest)
            .await
            .map_err(|e| {
                tracing::error!("failed to install module manifest for `{module_name}`: {e}");
                EX_UNAVAILABLE
            })?;

        if flags.verbose > 0 {
            cprintln!("<s,g>✓</> Installed module `{module_name}`.");
        }
    }
    Ok(())
}
