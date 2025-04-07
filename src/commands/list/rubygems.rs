// This is free and unencumbered software released into the public domain.

use super::http::http_client;
use reqwest::Error;
use serde::{Deserialize, Serialize};

/// Fetches JSON metadata for the current version of the `asimov-modules` gem.
pub async fn fetch_current_modules() -> Result<String, Error> {
    fetch_modules("25.0.0.dev.0").await // FIXME
}

/// Fetches JSON metadata for a specific version of the `asimov-modules` gem.
pub async fn fetch_modules(version: &str) -> Result<String, Error> {
    let url = format!(
        "https://rubygems.org/api/v2/rubygems/asimov-modules/versions/{}.json",
        version
    );
    http_client().get(&url).send().await?.text().await
}

/// Parses JSON metadata for the `asimov-modules` gem and extracts module names
/// from its runtime dependencies, removing the "asimov-" prefix and "-module"
/// suffix.
pub fn extract_module_names(json_str: impl AsRef<str>) -> serde_json::Result<Vec<String>> {
    let gem_info: GemInfo = serde_json::from_str(json_str.as_ref())?;

    let module_names = gem_info
        .dependencies
        .runtime
        .iter()
        .filter_map(|dep| {
            let name = &dep.name;

            // Handle the special case of "asimov-module" separately:
            if name == "asimov-module" {
                return None;
            }

            // Check if the dependency name has the expected prefix and suffix:
            if name.starts_with("asimov-") && name.ends_with("-module") {
                // Extract the middle part by removing prefix and suffix:
                let prefix_len = "asimov-".len();
                let suffix_len = "-module".len();
                let module_name = &name[prefix_len..name.len() - suffix_len];
                Some(module_name.to_string())
            } else {
                None
            }
        })
        .collect();

    Ok(module_names)
}

#[derive(Deserialize, Serialize, Debug)]
struct Dependency {
    name: String,
    requirements: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct Dependencies {
    development: Vec<Dependency>,
    runtime: Vec<Dependency>,
}

#[derive(Deserialize, Serialize, Debug)]
struct GemInfo {
    dependencies: Dependencies,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_asimov_module_names() {
        let json = r#"{
            "dependencies": {
                "development": [
                    {
                        "name": "rake",
                        "requirements": ">= 13"
                    }
                ],
                "runtime": [
                    {
                        "name": "asimov-anthropic-module",
                        "requirements": ">= 0"
                    },
                    {
                        "name": "asimov-chromium-module",
                        "requirements": ">= 0"
                    },
                    {
                        "name": "asimov-goodreads-module",
                        "requirements": ">= 0"
                    },
                    {
                        "name": "asimov-module",
                        "requirements": ">= 25.0.0.dev"
                    },
                    {
                        "name": "asimov-netscape-module",
                        "requirements": ">= 0"
                    },
                    {
                        "name": "asimov-openai-module",
                        "requirements": ">= 0"
                    },
                    {
                        "name": "asimov-rdf-module",
                        "requirements": ">= 0"
                    },
                    {
                        "name": "asimov-rdfs-module",
                        "requirements": ">= 0"
                    },
                    {
                        "name": "asimov-xsd-module",
                        "requirements": ">= 0"
                    }
                ]
            }
        }"#;

        let result = extract_module_names(json).unwrap();
        assert_eq!(
            result,
            vec![
                "anthropic",
                "chromium",
                "goodreads",
                "netscape",
                "openai",
                "rdf",
                "rdfs",
                "xsd"
            ]
        );
    }
}
