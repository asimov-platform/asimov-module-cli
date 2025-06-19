// This is free and unencumbered software released into the public domain.

use crate::{StandardOptions, SysexitsError, registry};
use asimov_env::paths::asimov_root;
use color_print::{ceprintln, cprint, cprintln};

#[tokio::main]
pub async fn list(flags: &StandardOptions) -> Result<(), SysexitsError> {
    let module_dir_path = asimov_root().join("modules");
    if module_dir_path.exists() && module_dir_path.is_dir() {
        let mut module_dir = tokio::fs::read_dir(module_dir_path)
            .await
            .inspect_err(|e| {
                ceprintln!("<s,r>error:</> failed to read module manifest directory: {e}")
            })?;
        loop {
            match module_dir.next_entry().await {
                Ok(None) => break,
                Ok(Some(entry)) => {
                    let filename = entry.file_name().to_string_lossy().to_string();
                    let name = filename.trim_end_matches(".yaml").trim_end_matches(".yml");
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

    for module in registry::fetch_modules().await? {
        let is_installed = module.is_installed()?;
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
