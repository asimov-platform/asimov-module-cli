// This is free and unencumbered software released into the public domain.

mod crates;
mod http;
mod pypi;
mod rubygems;

use crate::{StandardOptions, SysexitsError};
use tokio::task;

#[tokio::main]
pub async fn list(_flags: &StandardOptions) -> Result<(), SysexitsError> {
    // Spawn tasks to fetch module package metadata:
    let rust_task = task::spawn(async {
        let result = crates::fetch_current_modules()
            .await
            .expect("Fetch Rust module metadata");
        crates::extract_module_names(result)
    });
    let ruby_task = task::spawn(async {
        let result = rubygems::fetch_current_modules()
            .await
            .expect("Fetch Ruby module metadata");
        rubygems::extract_module_names(result)
    });
    let python_task = task::spawn(async {
        let result = pypi::fetch_current_modules()
            .await
            .expect("Fetch Python module metadata");
        pypi::extract_module_names(result)
    });

    // Await all tasks; note the double ?? to handle both `JoinError` and the
    // `Result` from the task:
    let rust_modules = rust_task.await??;
    let ruby_modules = ruby_task.await??;
    let python_modules = python_task.await??;

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
