// This is free and unencumbered software released into the public domain.

use crate::{
    StandardOptions,
    SysexitsError::{self, *},
};
use asimov_module::{resolve::Resolver, url::normalize_url};
use color_print::cprintln;

#[tokio::main]
pub async fn resolve(url: impl AsRef<str>, _flags: &StandardOptions) -> Result<(), SysexitsError> {
    let registry = asimov_registry::Registry::default();

    let manifests = registry
        .enabled_modules()
        .await
        .map_err(|e| {
            tracing::error!("failed to get installed modules: {e}");
            EX_UNAVAILABLE
        })?
        .into_iter()
        .map(|manifest| manifest.manifest);

    let resolver = Resolver::try_from_iter(manifests)
        .inspect_err(|e| tracing::error!("failed to create resolver: {e}"))?;

    let url = url.as_ref().to_string();
    let url = normalize_url(&url)
        .inspect_err(|e| {
            tracing::error!("Proceeding with given unmodified URL, normalization failed: {e}, ")
        })
        .unwrap_or(url);

    let modules = resolver
        .resolve(&url)
        .inspect_err(|e| tracing::error!("failed to resolve modules for URL `{url}`: {e}"))?;

    for module in modules {
        cprintln!("{}", module.name);
    }

    Ok(())
}
