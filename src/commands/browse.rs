// This is free and unencumbered software released into the public domain.

use asimov_env::paths::asimov_root;
use asimov_module::models::ModuleManifest;
use color_print::ceprintln;

use crate::{
    StandardOptions,
    SysexitsError::{self, *},
    registry,
};

#[tokio::main]
pub async fn browse(
    module_name: impl AsRef<str>,
    flags: &StandardOptions,
) -> Result<(), SysexitsError> {
    let module_name = module_name.as_ref();

    let manifest_path = asimov_root()
        .join("modules")
        .join(format!("{module_name}.yaml"));

    // try installed modules
    if let Ok(manifest) = tokio::fs::read(&manifest_path).await.inspect_err(|e| {
        if flags.verbose > 1 {
            tracing::warn!("install not found for `{module_name}`, failed to read manifest: {e}")
        }
    }) {
        let manifest: ModuleManifest = serde_yml::from_slice(&manifest).map_err(|e| {
            tracing::error!("failed parse manifest for module `{module_name}`: {e}");
            EX_UNAVAILABLE
        })?;

        let mut links = manifest.links;
        if let Some(link) = links.first() {
            open::that(link)
                .inspect_err(|e| tracing::error!("failed to open URL '{link}': {e}"))?;
            return Ok(());
        }
    }

    // try modules in registries
    if let Some(module) = registry::fetch_module(module_name).await {
        open::that(&module.url)
            .inspect_err(|e| tracing::error!("failed to open URL '{}': {e}", module.url))?;
        return Ok(());
    };

    eprintln!("unknown module: {}", module_name);
    Err(EX_UNAVAILABLE)
}
