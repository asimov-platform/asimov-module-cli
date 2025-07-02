// This is free and unencumbered software released into the public domain.

use crate::{
    StandardOptions,
    SysexitsError::{self, *},
    registry,
};
use asimov_env::paths::asimov_root;
use asimov_module::models::ModuleManifest;
use color_print::ceprintln;

#[tokio::main]
pub async fn link(
    module_name: impl AsRef<str>,
    _flags: &StandardOptions,
) -> Result<(), SysexitsError> {
    let module_name = module_name.as_ref();

    let manifest_path = asimov_root()
        .join("modules")
        .join(format!("{module_name}.yaml"));

    // try installed modules
    if let Ok(manifest) = tokio::fs::read(&manifest_path).await.inspect_err(|e| {
        tracing::warn!("install not found for `{module_name}`, failed to read manifest: {e}")
    }) {
        let manifest: ModuleManifest = serde_yml::from_slice(&manifest).map_err(|e| {
            tracing::error!("failed parse manifest for module `{module_name}`: {e}");
            EX_UNAVAILABLE
        })?;

        let mut links = manifest.links;
        crate::sort_links(&manifest.name, &mut links);

        for link in links {
            println!("{link}");
        }

        return Ok(());
    }

    // try modules in registries
    if let Some(module) = registry::fetch_module(module_name).await {
        println!("{}", module.url);
        return Ok(());
    };

    eprintln!("unknown module: {}", module_name);
    Err(EX_UNAVAILABLE)
}
