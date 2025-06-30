// This is free and unencumbered software released into the public domain.

use crate::{StandardOptions, SysexitsError, registry};

pub fn disable(module_names: Vec<String>, _flags: &StandardOptions) -> Result<(), SysexitsError> {
    for module_name in module_names {
        if !registry::is_enabled(&module_name) {
            continue; // skip already disabled modules
        }
        // TODO
    }
    Ok(())
}
