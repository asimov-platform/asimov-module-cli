// This is free and unencumbered software released into the public domain.

use asimov_env::paths::asimov_root;
use clientele::{
    StandardOptions,
    SysexitsError::{self, *},
};
use color_print::ceprintln;

#[tokio::main]
pub async fn config(module_name: String, flags: &StandardOptions) -> Result<(), SysexitsError> {
    let conf_bin = asimov_root()
        .join("libexec")
        .join(format!("asimov-{module_name}-configurator"));

    if !tokio::fs::try_exists(&conf_bin).await.unwrap_or(false) {
        ceprintln!("<s,r>error:</> module `{module_name}` has no configurator to run");
        return Err(EX_UNAVAILABLE);
    }

    Ok(())
}
