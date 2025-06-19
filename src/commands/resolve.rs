// This is free and unencumbered software released into the public domain.

use std::{error::Error, path::PathBuf};

use crate::{
    StandardOptions, SysexitsError,
    registry::{self, ModuleMetadata},
};
use asimov_env::paths::asimov_root;
use asimov_module::{models::ModuleManifest, resolve::ResolverBuilder};
use color_print::{ceprintln, cprint, cprintln};

#[tokio::main]
pub async fn resolve(url: impl AsRef<str>, _flags: &StandardOptions) -> Result<(), SysexitsError> {
    let mut builder = ResolverBuilder::new();

    let dir = std::fs::read_dir(asimov_root().join("modules")).inspect_err(|e| {
        ceprintln!("<r,s>error:</> failed to read module manifest directory: {e}")
    })?;

    for entry in dir {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let path = entry.path();
        let filename = entry.file_name();
        let filename = filename.to_string_lossy();
        if !filename.ends_with(".yaml") && !filename.ends_with(".yml") {
            continue;
        }
        let file = std::fs::File::open(&path)?;
        let manifest: ModuleManifest = serde_yml::from_reader(file).map_err(|e| {
            ceprintln!(
                "<s,y>warning:</> skipping invalid module manifest at `{}`: {e}",
                path.display()
            );
            SysexitsError::EX_UNAVAILABLE
        })?;
        builder.insert_manifest(&manifest)?;
    }

    let resolver = builder.build()?;

    let modules = resolver.resolve(url.as_ref())?;
    for module in modules {
        // let Some(module) = crate::registry::fetch_module(&module.name).await else {
        //     continue;
        // };

        // if module.is_installed()? {
        //     cprint!("<g,s>✓</> ");
        // } else {
        //     cprint!("<r,s>✗</> ");
        // }
        // cprintln!("{}", module.name);

        cprintln!("{}", module.name);
    }

    Ok(())
}
