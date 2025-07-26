// This is free and unencumbered software released into the public domain.

use crate::{
    StandardOptions,
    SysexitsError::{self, *},
};
use asimov_module::resolve::Resolver;
use color_print::cprintln;

#[tokio::main]
pub async fn resolve(url: impl AsRef<str>, _flags: &StandardOptions) -> Result<(), SysexitsError> {
    let installer = asimov_installer::Installer::default();

    let manifests = installer
        .enabled_modules()
        .await
        .map_err(|e| {
            tracing::error!("failed to read module manifest directory: {e}");
            EX_UNAVAILABLE
        })?
        .into_iter()
        .map(|manifest| manifest.manifest);

    let resolver = Resolver::try_from_iter(manifests)
        .inspect_err(|e| tracing::error!("failed to create resolver: {e}"))?;

    let modules = resolver.resolve(url.as_ref()).inspect_err(|e| {
        tracing::error!("failed to resolve modules for URL `{}`: {e}", url.as_ref())
    })?;

    for module in modules {
        cprintln!("{}", module.name);
    }

    Ok(())
}
