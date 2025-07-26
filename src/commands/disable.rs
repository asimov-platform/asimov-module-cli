// This is free and unencumbered software released into the public domain.

use crate::{
    StandardOptions,
    SysexitsError::{self, *},
};
use color_print::cprintln;

#[tokio::main]
pub async fn disable(
    module_names: Vec<String>,
    flags: &StandardOptions,
) -> Result<(), SysexitsError> {
    let installer = asimov_installer::Installer::default();
    for module_name in module_names {
        if flags.verbose > 1 {
            cprintln!("<s,c>»</> Disabling module `{module_name}`...");
        }

        installer.disable_module(&module_name).await.map_err(|e| {
            tracing::error!("failed to disable module `{module_name}`: {e}");
            EX_UNAVAILABLE
        })?;

        if flags.verbose > 0 {
            cprintln!("<s,g>✓</> Disabled module `{module_name}`.");
        }
    }
    Ok(())
}
