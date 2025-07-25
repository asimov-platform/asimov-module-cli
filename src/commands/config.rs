// This is free and unencumbered software released into the public domain.

use asimov_env::paths::asimov_root;
use clientele::{
    StandardOptions,
    SysexitsError::{self, *},
};
use color_print::ceprintln;
use std::io::{BufRead, Write};

#[tokio::main]
pub async fn config(
    module_name: String,
    mut args: &[String],
    _flags: &StandardOptions,
) -> Result<(), SysexitsError> {
    let installer = asimov_installer::Installer::default();
    let manifest = installer
        .manifest(&module_name)
        .await
        .map_err(|e| {
            tracing::error!("failed to read manifest for module `{module_name}`: {e}");
            if let asimov_installer::error::ManifestError::NotInstalled = e {
                ceprintln!(
                    "<s,dim>hint:</> Check if the module is installed with: `asimov module list`"
                );
            }
            EX_UNAVAILABLE
        })?
        .manifest;

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
            .and_then(|conf| conf.variables.first())
            .is_some_and(|var| var.name == args[0])
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

            let mut stdout = std::io::stdout().lock();
            let mut stdin = std::io::stdin().lock().lines();

            for var in conf_vars {
                let var_file = conf_dir.join(&var.name);

                let md = tokio::fs::metadata(&var_file).await;

                let current_value = tokio::fs::read_to_string(&var_file).await.ok();

                let is_required = var.default_value.is_none();

                let info_text = if current_value.is_some() {
                    "(press Enter to keep current)"
                } else if let Some(default_value) = &var.default_value {
                    &format!("(optional, default: `{default_value}`)")
                } else {
                    "(required)"
                };

                writeln!(&mut stdout, "Enter value for `{}` {info_text}", var.name)?;

                if let Some(current) = &current_value {
                    writeln!(&mut stdout, "Current value: `{}`", current.trim())?;
                }

                if let Some(desc) = &var.description {
                    writeln!(&mut stdout, "Description: {desc}");
                }

                let value = loop {
                    write!(&mut stdout, "> ")?;
                    stdout.flush()?;

                    match stdin.next() {
                        Some(Ok(line))
                            if current_value.is_none() && is_required && line.is_empty() =>
                        {
                            continue;
                        },
                        Some(Ok(line)) => break line,
                        Some(Err(e)) => return Err(e.into()),
                        None => return Err(EX_NOINPUT),
                    }
                };
                let value = value.trim();
                if value.is_empty() {
                    continue;
                }
                tokio::fs::write(&var_file, &value).await?;
            }

            let vars = manifest.read_variables(None).map_err(|e| {
                ceprintln!("<s,r>error:</> {e}");
                EX_UNAVAILABLE
            })?;
            writeln!(&mut stdout, "Configuration: ")?;
            for (name, value) in vars {
                writeln!(&mut stdout, "\t{name}: {value}")?;
            }
        } else if args.len() == 1 {
            // one arg, fetch the value

            let name = &args[0];
            if manifest
                .config
                .is_some_and(|conf| conf.variables.iter().any(|var| var.name == *name))
            {
                let var_file = conf_dir.join(name);
                if let Ok(current) = tokio::fs::read_to_string(&var_file).await {
                    println!("{}", current.trim());
                }
                // make args empty to not pass the same key to configurator
                args = &[];
            } else {
                ceprintln!("<s,r>error:</> unrecognized configuration variable key: `{name}`");
                return Err(EX_USAGE);
            }
        } else if args.len() % 2 == 0 {
            // pair(s) of (key,value), write into config file(s)

            let mut stdin = std::io::stdin().lock().lines();

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

                // confirm that user wants to overwrite
                if var_file.exists() {
                    let current = tokio::fs::read_to_string(&var_file).await?;
                    println!("Current value for `{name}`: {}", current.trim());
                    print!("Overwrite? [y/N]: ");

                    let input = stdin.next().ok_or(EX_NOINPUT)??;
                    if !input.trim().eq_ignore_ascii_case("y") {
                        continue;
                    }
                }

                tokio::fs::write(&var_file, &value).await?;
            }
        }
    }

    let configurator_name = format!("asimov-{module_name}-configurator");

    let provides_configurator = manifest.provides.programs.contains(&configurator_name);

    let conf_bin = asimov_root().join("libexec").join(configurator_name);

    let configurator_exists = tokio::fs::try_exists(&conf_bin).await.unwrap_or(false);

    if provides_configurator && configurator_exists {
        std::process::Command::new(&conf_bin)
            .args(args.iter())
            .stdin(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .status()
            .inspect_err(|e| ceprintln!("<s,r>error:</> failed to execute configurator: {e}"))?;
    }

    Ok(())
}
