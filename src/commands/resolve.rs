// This is free and unencumbered software released into the public domain.

use std::{error::Error, path::PathBuf};

use crate::{
    registry::{self, ModuleMetadata},
    StandardOptions, SysexitsError,
};
use asimov_env::paths::asimov_root;
use asimov_module::resolve::ResolverBuilder;
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
        let name = path.to_string_lossy();
        if !name.ends_with(".yaml") && !name.ends_with(".yml") {
            continue;
        }
        if let Err(e) = import_module_manifest(&mut builder, &path) {
            ceprintln!(
                "<y,s>warning:</> skipping module manifest at {}: {e}",
                path.display()
            )
        }
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

fn import_module_manifest(
    builder: &mut ResolverBuilder,
    path: &PathBuf,
) -> Result<(), Box<dyn Error>> {
    let manifest = std::fs::File::open(path)?;
    let module: serde_yml::Mapping = serde_yml::from_reader(manifest)?;
    let name = &module["name"]
        .as_str()
        .ok_or("Invalid module manifest: no name")?;

    if let Some(protocols) = module["handles"]["url_protocols"].as_sequence() {
        for protocol in protocols {
            builder.insert_protocol(
                name,
                protocol
                    .as_str()
                    .ok_or("Invalid module manifest: malformed handles.url_protocols")?,
            )?
        }
    }

    if let Some(prefixes) = module["handles"]["url_prefixes"].as_sequence() {
        for prefix in prefixes {
            builder.insert_prefix(
                name,
                prefix
                    .as_str()
                    .ok_or("Invalid module manifest: malformed handles.url_prefixes")?,
            )?
        }
    }

    if let Some(patterns) = module["handles"]["url_patterns"].as_sequence() {
        for pattern in patterns {
            builder.insert_pattern(
                name,
                pattern
                    .as_str()
                    .ok_or("Invalid module manifest: malformed handles.url_patterns")?,
            )?
        }
    }

    Ok(())
}
