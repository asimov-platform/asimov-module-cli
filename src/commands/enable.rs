// This is free and unencumbered software released into the public domain.

use crate::{registry, StandardOptions, SysexitsError};

pub fn enable(module_names: Vec<String>, _flags: &StandardOptions) -> Result<(), SysexitsError> {
    for module_name in module_names {
        if registry::is_enabled(&module_name) {
            continue; // skip already enabled modules
        }
        // TODO
    }
    Ok(())
}
