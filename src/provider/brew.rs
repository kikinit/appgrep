use std::path::PathBuf;
use std::process::Command;

use crate::app::{AppSource, Application};
use crate::provider::{AppProvider, ProviderError};

pub struct BrewProvider;

impl BrewProvider {
    pub fn new() -> Self {
        Self
    }

    fn has_brew() -> bool {
        Command::new("which")
            .arg("brew")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    fn brew_prefix() -> Option<String> {
        let output = Command::new("brew")
            .arg("--prefix")
            .output()
            .ok()?;

        if output.status.success() {
            Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            None
        }
    }

    pub fn parse_brew_json(json_str: &str) -> Vec<(String, Option<String>)> {
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(json_str);
        let mut formulae = Vec::new();

        if let Ok(value) = parsed {
            // brew info --json=v2 returns { "formulae": [...], "casks": [...] }
            if let Some(formula_array) = value.get("formulae").and_then(|v| v.as_array()) {
                for formula in formula_array {
                    let name = formula
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let desc = formula
                        .get("desc")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    if !name.is_empty() {
                        formulae.push((name, desc));
                    }
                }
            }
        }

        formulae
    }
}

impl AppProvider for BrewProvider {
    fn name(&self) -> &str {
        "brew"
    }

    fn is_available(&self) -> bool {
        Self::has_brew()
    }

    fn discover(&self) -> Result<Vec<Application>, ProviderError> {
        if !self.is_available() {
            return Ok(Vec::new());
        }

        let prefix = match Self::brew_prefix() {
            Some(p) => p,
            None => return Ok(Vec::new()),
        };

        let bin_dir = PathBuf::from(&prefix).join("bin");

        // Try to get descriptions from brew info --json=v2 --installed
        let json_output = Command::new("brew")
            .args(["info", "--json=v2", "--installed"])
            .output()
            .ok();

        let desc_map: std::collections::HashMap<String, Option<String>> =
            if let Some(ref out) = json_output {
                if out.status.success() {
                    let json_str = String::from_utf8_lossy(&out.stdout);
                    Self::parse_brew_json(&json_str)
                        .into_iter()
                        .collect()
                } else {
                    std::collections::HashMap::new()
                }
            } else {
                std::collections::HashMap::new()
            };

        // List installed formulae
        let list_output = Command::new("brew")
            .args(["list", "--formula"])
            .output()
            .map_err(ProviderError::Io)?;

        if !list_output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&list_output.stdout);
        let mut apps = Vec::new();

        for name in stdout.lines() {
            let name = name.trim();
            if name.is_empty() {
                continue;
            }

            let exec_path = bin_dir.join(name);
            if !exec_path.is_file() {
                continue;
            }

            let abs_path = exec_path.to_string_lossy().to_string();
            let description = desc_map.get(name).cloned().flatten();

            apps.push(Application {
                name: name.to_string(),
                exec_command: abs_path.clone(),
                source: AppSource::Brew,
                location: abs_path,
                icon: None,
                categories: vec!["Homebrew".to_string()],
                description,
            });
        }

        Ok(apps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_name() {
        let provider = BrewProvider::new();
        assert_eq!(provider.name(), "brew");
    }

    #[test]
    fn test_parse_brew_json_valid() {
        let json = r#"{
            "formulae": [
                {"name": "git", "desc": "Distributed revision control system"},
                {"name": "wget", "desc": "Internet file retriever"}
            ],
            "casks": []
        }"#;
        let formulae = BrewProvider::parse_brew_json(json);
        assert_eq!(formulae.len(), 2);
        assert_eq!(formulae[0].0, "git");
        assert_eq!(
            formulae[0].1,
            Some("Distributed revision control system".to_string())
        );
        assert_eq!(formulae[1].0, "wget");
    }

    #[test]
    fn test_parse_brew_json_empty() {
        let json = r#"{"formulae": [], "casks": []}"#;
        let formulae = BrewProvider::parse_brew_json(json);
        assert!(formulae.is_empty());
    }

    #[test]
    fn test_parse_brew_json_invalid() {
        let formulae = BrewProvider::parse_brew_json("not json");
        assert!(formulae.is_empty());
    }

    #[test]
    fn test_parse_brew_json_no_desc() {
        let json = r#"{
            "formulae": [
                {"name": "tool", "desc": null}
            ],
            "casks": []
        }"#;
        let formulae = BrewProvider::parse_brew_json(json);
        assert_eq!(formulae.len(), 1);
        assert_eq!(formulae[0].1, None);
    }
}
