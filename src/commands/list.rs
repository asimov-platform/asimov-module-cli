// This is free and unencumbered software released into the public domain.

mod crates;
mod http;
mod pypi;
mod rubygems;

use crate::{StandardOptions, SysexitsError};

#[tokio::main]
pub async fn list(_flags: &StandardOptions) -> Result<(), SysexitsError> {
    let rust_modules: Vec<String> = vec![]; // TODO

    let ruby_modules = rubygems::fetch_current_modules()
        .await
        .map(rubygems::extract_module_names)
        .expect("Parse Ruby modules")
        .expect("Fetch Ruby modules");

    let python_modules = pypi::fetch_current_modules()
        .await
        .map(pypi::extract_module_names)
        .expect("Parse Python modules")
        .expect("Fetch Python modules");

    let mut all_modules: Vec<String> = rust_modules
        .iter()
        .chain(ruby_modules.iter())
        .chain(python_modules.iter())
        .cloned()
        .collect();

    all_modules.sort();

    for module in all_modules {
        println!("{}", module);
    }

    Ok(())
}
