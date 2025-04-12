// This is free and unencumbered software released into the public domain.

use crate::{registry, StandardOptions, SysexitsError};

#[tokio::main]
pub async fn list(_flags: &StandardOptions) -> Result<(), SysexitsError> {
    for module in registry::fetch_modules().await? {
        println!("{}", module);
    }
    Ok(())
}
