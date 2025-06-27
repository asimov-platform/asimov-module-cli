// This is free and unencumbered software released into the public domain.

use crate::{StandardOptions, SysexitsError, registry};
use asimov_env::paths::asimov_root;
use color_print::{ceprintln, cprint, cprintln};

#[tokio::main]
pub async fn list(flags: &StandardOptions) -> Result<(), SysexitsError> {
    let module_dir_path = asimov_root().join("modules");

    let md = tokio::fs::metadata(&module_dir_path).await;
    if md.is_ok_and(|md| md.is_dir()) {
        let mut module_dir = tokio::fs::read_dir(module_dir_path)
            .await
            .inspect_err(|e| tracing::error!("failed to read module manifest directory: {e}"))?;
        loop {
            match module_dir.next_entry().await {
                Ok(None) => break,
                Ok(Some(entry)) => {
                    let Some(name) = entry
                        .path()
                        .file_stem()
                        .map(|stem| stem.to_string_lossy().to_string())
                    else {
                        continue;
                    };

                    if flags.verbose > 0 {
                        cprintln!("<s,g>✓</> {}\t\t", name);
                    } else {
                        cprintln!("<s,g>✓</> {}", name);
                    }
                }
                Err(e) => continue,
            }
        }
    }

    for module in registry::fetch_modules()
        .await
        .inspect_err(|e| tracing::error!("failed to fetch module registry: {e}"))?
    {
        let is_installed = module.is_installed().inspect_err(|e| {
            tracing::error!(
                "failed to check if module '{}' is installed: {e}",
                module.name,
            )
        })?;
        if is_installed {
            cprint!("<s><g>✓</></> ");
        } else {
            cprint!("<s><r>✗</></> ");
        }
        if flags.verbose > 0 {
            cprintln!("{}\t{}\t{}", module.name, module.version, module.r#type);
        } else {
            cprintln!("{}", module.name);
        }
    }
    Ok(())
}
