// This is free and unencumbered software released into the public domain.

use crate::{registry, StandardOptions, SysexitsError};

#[tokio::main]
pub async fn list(flags: &StandardOptions) -> Result<(), SysexitsError> {
    for module in registry::fetch_modules().await? {
        if flags.verbose > 0 {
            println!("{}\t{}\t{}", module.name, module.version, module.r#type);
        } else {
            println!("{}", module.name);
        }
    }
    Ok(())
}
