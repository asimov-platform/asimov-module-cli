// This is free and unencumbered software released into the public domain.

use crate::{
    StandardOptions,
    SysexitsError::{self, *},
};
use color_print::ceprintln;

#[tokio::main]
pub async fn link(
    module_name: impl AsRef<str>,
    _flags: &StandardOptions,
) -> Result<(), SysexitsError> {
    let installer = asimov_module::installer::Installer::default();

    let manifest = installer
        .manifest(module_name)
        .await
        .map_err(|e| {
            tracing::error!("failed to read module manifest: {e}");
            EX_UNAVAILABLE
        })?
        .manifest;

    let mut links = manifest.links;
    crate::sort_links(&manifest.name, &mut links);

    for link in links {
        println!("{link}");
    }

    Ok(())
}
