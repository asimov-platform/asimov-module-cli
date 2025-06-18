// This is free and unencumbered software released into the public domain.

#[tokio::main]
async fn main() {
    let module_name = "apify";
    let release = asimov_module_cli::registry::github::fetch_latest_release(module_name)
        .await
        .expect("Fetching latest release should succeed");
    asimov_module_cli::registry::github::install_from_github(module_name, &release, 2)
        .await
        .expect("Installing release should succeed");
}
