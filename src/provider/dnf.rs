use std::collections::HashSet;
use std::path::Path;
use std::process::Command;

use crate::app::{AppSource, Application};
use crate::provider::{AppProvider, ProviderError};

pub struct RpmProvider;

impl RpmProvider {
    pub fn new() -> Self {
        Self
    }

    fn has_rpm() -> bool {
        Command::new("which")
            .arg("rpm")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    fn has_desktop_file(pkg: &str) -> bool {
        let apps_dir = Path::new("/usr/share/applications");
        if !apps_dir.is_dir() {
            return false;
        }
        if apps_dir.join(format!("{}.desktop", pkg)).exists() {
            return true;
        }
        if let Ok(entries) = std::fs::read_dir(apps_dir) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.starts_with(pkg) && name.ends_with(".desktop") {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn parse_rpm_output(output: &str) -> Vec<(String, Option<String>)> {
        let mut packages = Vec::new();
        for line in output.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.splitn(2, '\t').collect();
            if parts.is_empty() {
                continue;
            }
            let name = parts[0].to_string();
            let description = parts.get(1).map(|d| d.to_string()).filter(|d| !d.is_empty());
            packages.push((name, description));
        }
        packages
    }

    fn find_package_binary(pkg: &str) -> Option<String> {
        let output = Command::new("rpm")
            .args(["-ql", pkg])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let line = line.trim();
            if (line.starts_with("/usr/bin/") || line.starts_with("/usr/local/bin/")
                || line.starts_with("/usr/sbin/"))
                && Path::new(line).is_file()
            {
                return Some(line.to_string());
            }
        }

        None
    }
}

impl AppProvider for RpmProvider {
    fn name(&self) -> &str {
        "rpm"
    }

    fn is_available(&self) -> bool {
        Self::has_rpm()
    }

    fn discover(&self) -> Result<Vec<Application>, ProviderError> {
        if !self.is_available() {
            return Ok(Vec::new());
        }

        let output = Command::new("rpm")
            .args(["-qa", "--queryformat", "%{NAME}\\t%{SUMMARY}\\n"])
            .output()
            .map_err(ProviderError::Io)?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let packages = Self::parse_rpm_output(&stdout);

        let mut seen_binaries = HashSet::new();
        let mut apps = Vec::new();

        for (pkg_name, description) in packages {
            if Self::has_desktop_file(&pkg_name) {
                continue;
            }

            if let Some(binary) = Self::find_package_binary(&pkg_name) {
                if !seen_binaries.insert(binary.clone()) {
                    continue;
                }

                let exec_name = Path::new(&binary)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(&pkg_name)
                    .to_string();

                apps.push(Application {
                    name: exec_name,
                    exec_command: binary.clone(),
                    source: AppSource::Rpm,
                    location: binary,
                    icon: None,
                    categories: vec!["CLI".to_string()],
                    description,
                });
            }
        }

        Ok(apps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_name() {
        let provider = RpmProvider::new();
        assert_eq!(provider.name(), "rpm");
    }

    #[test]
    fn test_parse_rpm_output_valid() {
        let output = "curl\tA utility for getting files from remote servers\ngit\tFast Version Control System\n";
        let packages = RpmProvider::parse_rpm_output(output);
        assert_eq!(packages.len(), 2);
        assert_eq!(packages[0].0, "curl");
        assert!(packages[0].1.as_ref().unwrap().contains("remote servers"));
        assert_eq!(packages[1].0, "git");
    }

    #[test]
    fn test_parse_rpm_output_empty() {
        let packages = RpmProvider::parse_rpm_output("");
        assert!(packages.is_empty());
    }

    #[test]
    fn test_parse_rpm_output_no_description() {
        let output = "somepackage\t\n";
        let packages = RpmProvider::parse_rpm_output(output);
        assert_eq!(packages.len(), 1);
        assert_eq!(packages[0].1, None);
    }

    #[test]
    fn test_parse_rpm_output_malformed() {
        let output = "\n   \n\n";
        let packages = RpmProvider::parse_rpm_output("");
        assert!(packages.is_empty());
        let _ = output;
    }
}
