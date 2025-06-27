// This is free and unencumbered software released into the public domain.

use crate::{
    StandardOptions,
    SysexitsError::{self, *},
    registry,
};

#[tokio::main]
pub async fn browse(
    module_name: impl AsRef<str>,
    _flags: &StandardOptions,
) -> Result<(), SysexitsError> {
    let module_name = module_name.as_ref();

    match registry::fetch_module(module_name).await {
        Some(module) => {
            open::that(&module.url)
                .inspect_err(|e| tracing::error!("failed to open URL '{}': {e}", module.url))?;
            Ok(())
        }
        None => {
            eprintln!("unknown module: {}", module_name);
            Err(EX_UNAVAILABLE)
        }
    }
}
