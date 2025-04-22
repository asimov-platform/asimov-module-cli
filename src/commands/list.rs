// This is free and unencumbered software released into the public domain.

use crate::{registry, StandardOptions, SysexitsError};
use color_print::{cprint, cprintln};

#[tokio::main]
pub async fn list(flags: &StandardOptions) -> Result<(), SysexitsError> {
    for module in registry::fetch_modules().await? {
        let is_installed = module.is_installed()?;
        if is_installed {
            cprint!("<s><g>✓</></> ");
        } else {
            cprint!("<s><r>✗</></> ");
        }
        if flags.verbose > 0 {
            println!("{}\t{}\t{}", module.name, module.version, module.r#type);
        } else {
            cprintln!("{}", module.name,);
        }
    }
    Ok(())
}
