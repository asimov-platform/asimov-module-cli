// This is free and unencumbered software released into the public domain.

use std::{error::Error, path::PathBuf};

use crate::{
    StandardOptions,
    SysexitsError::{self, *},
    registry::{self, ModuleMetadata},
};
use asimov_env::paths::asimov_root;
use asimov_module::{models::ModuleManifest, resolve::Resolver};
use color_print::{ceprintln, cprint, cprintln};

#[tokio::main]
pub async fn resolve(url: impl AsRef<str>, _flags: &StandardOptions) -> Result<(), SysexitsError> {
    let mut resolver = Resolver::new();

    let dir = std::fs::read_dir(asimov_root().join("modules"))
        .inspect_err(|e| tracing::error!("failed to read module manifest directory: {e}"))?;

    for entry in dir {
        let entry =
            entry.inspect_err(|e| tracing::error!("failed to read directory entry: {e}"))?;
        if !entry
            .file_type()
            .inspect_err(|e| {
                tracing::error!(
                    "failed to get file type for '{}': {e}",
                    entry.path().display()
                )
            })?
            .is_file()
        {
            continue;
        }
        let path = entry.path();
        let filename = entry.file_name();
        let filename = filename.to_string_lossy();
        if !filename.ends_with(".yaml") && !filename.ends_with(".yml") {
            continue;
        }
        let file = std::fs::File::open(&path).inspect_err(|e| {
            tracing::error!("failed to open manifest file '{}': {e}", path.display())
        })?;
        let manifest: ModuleManifest = serde_yml::from_reader(file).map_err(|e| {
            tracing::warn!(
                "skipping invalid module manifest at `{}`: {e}",
                path.display()
            );
            EX_UNAVAILABLE
        })?;
        resolver.insert_manifest(&manifest).inspect_err(|e| {
            tracing::error!("failed to insert manifest from '{}': {e}", path.display())
        })?;
    }

    let modules = resolver.resolve(url.as_ref()).inspect_err(|e| {
        tracing::error!("failed to resolve modules for URL '{}': {e}", url.as_ref())
    })?;

    for module in modules {
        cprintln!("{}", module.name);
    }

    Ok(())
}
