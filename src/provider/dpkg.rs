use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::app::{AppSource, Application};
use crate::provider::{AppProvider, ProviderError};

pub struct DpkgProvider;

impl DpkgProvider {
    pub fn new() -> Self {
        Self
    }

    fn has_dpkg_query() -> bool {
        Command::new("which")
            .arg("dpkg-query")
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
        // Check for <pkg>.desktop or <pkg>-*.desktop patterns
        if apps_dir.join(format!("{}.desktop", pkg)).exists() {
            return true;
        }
        // Check for any desktop file starting with the package name
        if let Ok(entries) = fs::read_dir(apps_dir) {
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

    fn find_package_binary(pkg: &str) -> Option<String> {
        // Check /var/lib/dpkg/info/<pkg>.list for binaries
        let list_file = format!("/var/lib/dpkg/info/{}.list", pkg);
        if let Ok(content) = fs::read_to_string(&list_file) {
            for line in content.lines() {
                let line = line.trim();
                if (line.starts_with("/usr/bin/") || line.starts_with("/usr/local/bin/")
                    || line.starts_with("/usr/sbin/"))
                    && Path::new(line).is_file()
                {
                    return Some(line.to_string());
                }
            }
        }

        // Also try arch-qualified list files (e.g., pkg:amd64.list)
        let info_dir = Path::new("/var/lib/dpkg/info");
        if info_dir.is_dir() {
            if let Ok(entries) = fs::read_dir(info_dir) {
                for entry in entries.flatten() {
                    if let Some(name) = entry.file_name().to_str() {
                        if name.starts_with(&format!("{}:", pkg)) && name.ends_with(".list") {
                            if let Ok(content) = fs::read_to_string(entry.path()) {
                                for line in content.lines() {
                                    let line = line.trim();
                                    if (line.starts_with("/usr/bin/")
                                        || line.starts_with("/usr/local/bin/")
                                        || line.starts_with("/usr/sbin/"))
                                        && Path::new(line).is_file()
                                    {
                                        return Some(line.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        None
    }

    pub fn parse_dpkg_output(output: &str) -> Vec<(String, Option<String>)> {
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
}

impl AppProvider for DpkgProvider {
    fn name(&self) -> &str {
        "dpkg"
    }

    fn is_available(&self) -> bool {
        Self::has_dpkg_query()
    }

    fn discover(&self) -> Result<Vec<Application>, ProviderError> {
        if !self.is_available() {
            return Ok(Vec::new());
        }

        let output = Command::new("dpkg-query")
            .args(["-W", "-f=${Package}\\t${binary:Summary}\\n"])
            .output()
            .map_err(ProviderError::Io)?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let packages = Self::parse_dpkg_output(&stdout);

        let mut seen_binaries = HashSet::new();
        let mut apps = Vec::new();

        for (pkg_name, description) in packages {
            // Skip packages that have a .desktop file (already covered by desktop provider)
            if Self::has_desktop_file(&pkg_name) {
                continue;
            }

            // Find a binary for this package
            if let Some(binary) = Self::find_package_binary(&pkg_name) {
                // Skip if we've already seen this binary
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
                    source: AppSource::Dpkg,
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
        let provider = DpkgProvider::new();
        assert_eq!(provider.name(), "dpkg");
    }

    #[test]
    fn test_parse_dpkg_output_valid() {
        let output = "curl\tcommand line tool for transferring data\ngit\tfast, scalable, distributed revision control system\n";
        let packages = DpkgProvider::parse_dpkg_output(output);
        assert_eq!(packages.len(), 2);
        assert_eq!(packages[0].0, "curl");
        assert_eq!(
            packages[0].1,
            Some("command line tool for transferring data".to_string())
        );
        assert_eq!(packages[1].0, "git");
    }

    #[test]
    fn test_parse_dpkg_output_empty() {
        let packages = DpkgProvider::parse_dpkg_output("");
        assert!(packages.is_empty());
    }

    #[test]
    fn test_parse_dpkg_output_no_description() {
        let output = "somepackage\t\n";
        let packages = DpkgProvider::parse_dpkg_output(output);
        assert_eq!(packages.len(), 1);
        assert_eq!(packages[0].0, "somepackage");
        assert_eq!(packages[0].1, None);
    }

    #[test]
    fn test_parse_dpkg_output_malformed() {
        let output = "\n\n  \n";
        let packages = DpkgProvider::parse_dpkg_output(output);
        assert!(packages.is_empty());
    }
}
