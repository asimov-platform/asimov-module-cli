// This is free and unencumbered software released into the public domain.

use crate::{
    StandardOptions,
    SysexitsError::{self, *},
};
use color_print::cprintln;

#[tokio::main]
pub async fn list(flags: &StandardOptions) -> Result<(), SysexitsError> {
    let installer = asimov_installer::Installer::default();
    let modules = installer.installed_modules().await.map_err(|e| {
        tracing::error!("failed to read installed modules: {e}");
        EX_UNAVAILABLE
    })?;

    for module in modules {
        let name = module.manifest.name;

        let is_enabled = if installer.is_module_enabled(&name).await.map_err(|e| {
            tracing::error!("failed to check if module is enabled: {e}");
            EX_UNAVAILABLE
        })? {
            color_print::cstr!("<s,g>enabled</>")
        } else {
            color_print::cstr!("<s,r>disabled</>")
        };

        if flags.verbose > 0 {
            cprintln!("<s,g>✓</> {}\t{}", name, is_enabled);
        } else {
            cprintln!("<s,g>✓</> {}", name);
        }
    }
    Ok(())
}
