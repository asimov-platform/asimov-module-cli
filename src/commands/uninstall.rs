// This is free and unencumbered software released into the public domain.

use crate::{registry, StandardOptions, SysexitsError};

pub fn uninstall(module_names: Vec<String>, _flags: &StandardOptions) -> Result<(), SysexitsError> {
    let mut modules_to_uninstall: Vec<String> = vec![];

    for module_name in module_names {
        if !registry::is_installed(&module_name) {
            continue; // skip not installed modules
        }
        modules_to_uninstall.push(module_name.clone());
    }

    for _module_name in modules_to_uninstall {
        // TODO
    }

    Ok(())
}
