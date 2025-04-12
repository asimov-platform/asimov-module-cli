// This is free and unencumbered software released into the public domain.

use crate::{registry, StandardOptions, SysexitsError};

#[tokio::main]
pub async fn link(
    module_name: impl AsRef<str>,
    _flags: &StandardOptions,
) -> Result<(), SysexitsError> {
    let module_name = module_name.as_ref();
    match registry::fetch_module(module_name).await {
        Some(module) => {
            println!("{}", module.url);
            Ok(())
        }
        None => {
            eprintln!("unknown module: {}", module_name);
            Err(SysexitsError::EX_UNAVAILABLE)
        }
    }
}
