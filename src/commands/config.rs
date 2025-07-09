// This is free and unencumbered software released into the public domain.

use std::io::{BufRead, Write};

use asimov_env::paths::asimov_root;
use asimov_module::models::ModuleManifest;
use clientele::{
    StandardOptions,
    SysexitsError::{self, *},
};
use color_print::ceprintln;

#[tokio::main]
pub async fn config(
    module_name: String,
    args: &[String],
    flags: &StandardOptions,
) -> Result<(), SysexitsError> {
    let manifest = ModuleManifest::read_manifest(&module_name).inspect_err(|e| {
        ceprintln!("<s,r>error:</> failed to read manifest for module `{module_name}`: {e}")
    })?;

    let mod_has_conf_vars = manifest
        .config
        .as_ref()
        .is_some_and(|conf| !conf.variables.is_empty());

    let first_arg_is_key = if args.is_empty() {
        false
    } else {
        manifest
            .config
            .as_ref()
            .is_some_and(|conf| conf.variables.iter().any(|var| var.name == args[0]))
    };

    if mod_has_conf_vars && (args.is_empty() || first_arg_is_key) {
        let profile = "default"; // TODO

        let conf_dir = asimov_root()
            .join("configs")
            .join(profile)
            .join(&module_name);

        tokio::fs::create_dir_all(&conf_dir).await.inspect_err(|e| {
            ceprintln!("<s,r>error:</> failed to create configuration directory for module `{module_name}`: {e}");
        })?;

        if args.is_empty() {
            // interactively prompt for each value in the config

            let mut prompt_for_value = {
                let mut stdout = std::io::stdout().lock();
                let mut stdin = std::io::stdin().lock().lines();
                move |name: &str, desc: Option<&str>| {
                    write!(&mut stdout, "Give an value for `{name}`")?;
                    if let Some(desc) = desc {
                        write!(&mut stdout, " (description: `{desc}`)")?;
                    }
                    writeln!(&mut stdout)?;

                    loop {
                        write!(&mut stdout, "> ")?;
                        stdout.flush()?;

                        if let Some(line) = stdin.next() {
                            return line;
                        }
                    }
                }
            };

            for var in manifest.config.unwrap_or_default().variables {
                let var_file = conf_dir.join(&var.name);

                let md = tokio::fs::metadata(&var_file).await;
                if md.is_err_and(|err| err.kind() == std::io::ErrorKind::NotFound) {
                    let value = prompt_for_value(&var.name, var.description.as_deref())?;
                    tokio::fs::write(&var_file, &value).await?;
                }
            }
        } else if args.len() == 1 {
            // one arg, fetch the value

            let name = &args[0];
            if !manifest
                .config
                .is_some_and(|conf| conf.variables.iter().any(|var| var.name == *name))
            {
                return Ok(());
            }
            let var_file = conf_dir.join(name);
            let value = tokio::fs::read_to_string(&var_file).await?;
            println!("{value}")
        } else if args.len().is_multiple_of(2) {
            // pair(s) of (key,value), write into config file(s)

            let mut chunks = args.chunks_exact(2);
            while let Some([name, value]) = chunks.next() {
                let var_file = conf_dir.join(name);
                tokio::fs::write(&var_file, &value).await?;
            }
        }
    }

    let provides_configurator = manifest
        .provides
        .programs
        .iter()
        .any(|program| *program == format!("asimov-{module_name}-configurator"));

    let conf_bin = asimov_root()
        .join("libexec")
        .join(format!("asimov-{module_name}-configurator"));

    let configurator_exists = tokio::fs::try_exists(&conf_bin).await.unwrap_or(false);

    if provides_configurator && configurator_exists {
        std::process::Command::new(&conf_bin)
            .args(args)
            .stdin(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .status();
    }

    Ok(())
}
