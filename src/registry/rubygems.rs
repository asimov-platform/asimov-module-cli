// This is free and unencumbered software released into the public domain.

use super::{http::http_client, ModuleMetadata, ModuleType};
use reqwest::Error;
use serde::{Deserialize, Serialize};

/// Fetches JSON metadata for the current `asimov-modules` gem.
pub async fn fetch_current_modules() -> Result<String, Error> {
    fetch_modules("25.0.0.dev.0").await // FIXME
}

/// Fetches JSON metadata for a specific `asimov-modules` gem version.
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
pub fn extract_module_names(json_str: impl AsRef<str>) -> serde_json::Result<Vec<ModuleMetadata>> {
    let gem_info: GemInfo = serde_json::from_str(json_str.as_ref())?;

    let module_names = gem_info
        .dependencies
        .runtime
        .iter()
        .filter_map(|dep| {
            let dep_name = &dep.name;

            // Handle the special case of "asimov-module" separately:
            if dep_name == "asimov-module" {
                return None;
            }

            // Check if the dependency name has the expected prefix and suffix:
            if dep_name.starts_with("asimov-") && dep_name.ends_with("-module") {
                // Strip the "asimov-" prefix and "-module" suffix:
                let mod_name = dep_name.strip_prefix("asimov-")?.strip_suffix("-module")?;

                Some(ModuleMetadata {
                    name: mod_name.to_string(),
                    version: gem_info.version.clone(),
                    r#type: ModuleType::Ruby,
                    url: format!("https://rubygems.org/gems/{}", dep_name),
                })
            } else {
                None
            }
        })
        .collect();

    Ok(module_names)
}

#[derive(Deserialize, Serialize, Debug)]
struct GemInfo {
    version: String,
    dependencies: Dependencies,
}

#[derive(Deserialize, Serialize, Debug)]
struct Dependencies {
    development: Vec<Dependency>,
    runtime: Vec<Dependency>,
}

#[derive(Deserialize, Serialize, Debug)]
struct Dependency {
    name: String,
    requirements: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_module_names() {
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
