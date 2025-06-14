// This is free and unencumbered software released into the public domain.

use asimov_module_cli::registry::{ModuleMetadata, ModuleType};

#[tokio::test]
async fn main() {
    let md = ModuleMetadata {
        name: "apify".into(),
        version: "0.1.1".into(),
        r#type: ModuleType::Rust,
        url: "https://github.com/asimov-modules/asimov-apify-module".into(), // does not matter, it's not used
    };

    let res = asimov_module_cli::registry::github::install_from_github(&md, 2)
        .await
        .unwrap();
    assert_eq!(res.code(), Some(0));
}
