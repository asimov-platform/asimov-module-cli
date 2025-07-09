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
    mut args: &[String],
    flags: &StandardOptions,
) -> Result<(), SysexitsError> {
    let manifest = ModuleManifest::read_manifest(&module_name).inspect_err(|e| {
        ceprintln!("<s,r>error:</> failed to read manifest for module `{module_name}`: {e}")
    })?;

    let conf_vars = manifest
        .config
        .as_ref()
        .map(|c| c.variables.as_slice())
        .unwrap_or_default();

    let first_arg_is_key = if args.is_empty() {
        false
    } else {
        manifest
            .config
            .as_ref()
            .is_some_and(|conf| conf.variables.iter().any(|var| var.name == args[0]))
    };

    if !conf_vars.is_empty() && (args.is_empty() || first_arg_is_key) {
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

            for var in conf_vars {
                let var_file = conf_dir.join(&var.name);

                let md = tokio::fs::metadata(&var_file).await;
                if md.is_err_and(|err| err.kind() == std::io::ErrorKind::NotFound) {
                    let value = prompt_for_value(&var.name, var.description.as_deref())?;
                    if value.is_empty() {
                        continue;
                    }
                    tokio::fs::write(&var_file, &value).await?;
                }
            }

            let vars = manifest.read_variables(None).map_err(|e| {
                ceprintln!("<s,r>error:</> {e}");
                EX_UNAVAILABLE
            })?;
            println!("Configuration: ");
            for (name, value) in vars {
                println!("\t{name}: {value}");
            }
        } else if args.len() == 1 {
            // one arg, fetch the value

            let name = &args[0];
            if manifest
                .config
                .is_some_and(|conf| conf.variables.iter().any(|var| var.name == *name))
            {
                let var_file = conf_dir.join(name);
                let value = tokio::fs::read_to_string(&var_file).await?;
                println!("{name}: {value}")
            }
        } else if args.len().is_multiple_of(2) {
            // pair(s) of (key,value), write into config file(s)

            loop {
                // split a 2-tuple from args
                let Some(([name, value], rest)) = args.split_first_chunk() else {
                    break;
                };
                // must be a known configuration variable, otherwise stop
                if !conf_vars.iter().any(|var| var.name == *name) {
                    break;
                }
                // re-slice the args
                args = rest;
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
            .args(args.iter())
            .stdin(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .status();
    }

    Ok(())
}
