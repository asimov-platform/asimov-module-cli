// This is free and unencumbered software released into the public domain.

pub mod crates;
pub mod http;
pub mod pypi;
pub mod rubygems;

use crate::{registry, SysexitsError};
use tokio::task;

pub async fn fetch_modules() -> Result<Vec<String>, SysexitsError> {
    // Spawn tasks to fetch module package metadata:
    let rust_task = task::spawn(async {
        let result = registry::crates::fetch_current_modules()
            .await
            .expect("Fetch Rust module metadata");
        registry::crates::extract_module_names(result)
    });
    let ruby_task = task::spawn(async {
        let result = registry::rubygems::fetch_current_modules()
            .await
            .expect("Fetch Ruby module metadata");
        registry::rubygems::extract_module_names(result)
    });
    let python_task = task::spawn(async {
        let result = registry::pypi::fetch_current_modules()
            .await
            .expect("Fetch Python module metadata");
        registry::pypi::extract_module_names(result)
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

    Ok(all_modules)
}
