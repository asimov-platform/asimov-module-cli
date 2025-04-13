// This is free and unencumbered software released into the public domain.

use crate::{StandardOptions, SysexitsError};

pub fn uninstall(module_names: Vec<String>, _flags: &StandardOptions) -> Result<(), SysexitsError> {
    for _module_name in module_names {
        // TODO
    }
    Ok(())
}
