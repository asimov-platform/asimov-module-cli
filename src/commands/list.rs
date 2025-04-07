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

    let python_modules: Vec<String> = vec![]; // TODO

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
