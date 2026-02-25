use std::collections::HashSet;
use std::path::Path;
use std::process::Command;

use crate::app::{AppSource, Application};
use crate::provider::{AppProvider, ProviderError};

pub struct PacmanProvider;

impl PacmanProvider {
    pub fn new() -> Self {
        Self
    }

    fn has_pacman() -> bool {
        Command::new("which")
            .arg("pacman")
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

    pub fn parse_pacman_info(output: &str) -> Vec<(String, Option<String>)> {
        let mut packages = Vec::new();
        let mut current_name: Option<String> = None;
        let mut current_desc: Option<String> = None;

        for line in output.lines() {
            if let Some(name) = line.strip_prefix("Name            : ") {
                // Save previous package
                if let Some(ref name) = current_name {
                    packages.push((name.clone(), current_desc.take()));
                }
                current_name = Some(name.trim().to_string());
                current_desc = None;
            } else if let Some(desc) = line.strip_prefix("Description     : ") {
                current_desc = Some(desc.trim().to_string());
            }
        }

        // Save last package
        if let Some(name) = current_name {
            packages.push((name, current_desc));
        }

        packages
    }

    fn find_package_binary(pkg: &str) -> Option<String> {
        // First check if /usr/bin/<pkg> exists directly
        let direct = format!("/usr/bin/{}", pkg);
        if Path::new(&direct).is_file() {
            return Some(direct);
        }

        // Fall back to pacman -Ql
        let output = Command::new("pacman")
            .args(["-Ql", pkg])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            // Format: "pkgname /usr/bin/something"
            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            if parts.len() == 2 {
                let path = parts[1].trim();
                if (path.starts_with("/usr/bin/") || path.starts_with("/usr/local/bin/"))
                    && Path::new(path).is_file()
                {
                    return Some(path.to_string());
                }
            }
        }

        None
    }
}

impl AppProvider for PacmanProvider {
    fn name(&self) -> &str {
        "pacman"
    }

    fn is_available(&self) -> bool {
        Self::has_pacman()
    }

    fn discover(&self) -> Result<Vec<Application>, ProviderError> {
        if !self.is_available() {
            return Ok(Vec::new());
        }

        // Get list of installed packages
        let list_output = Command::new("pacman")
            .args(["-Qq"])
            .output()
            .map_err(ProviderError::Io)?;

        if !list_output.status.success() {
            return Ok(Vec::new());
        }

        let pkg_list = String::from_utf8_lossy(&list_output.stdout);
        let pkg_names: Vec<&str> = pkg_list.lines().filter(|l| !l.is_empty()).collect();

        // Get info for all packages at once
        let mut info_cmd = Command::new("pacman");
        info_cmd.arg("-Qi");
        for name in &pkg_names {
            info_cmd.arg(name);
        }
        let info_output = info_cmd
            .output()
            .map_err(ProviderError::Io)?;

        let info_stdout = String::from_utf8_lossy(&info_output.stdout);
        let packages = Self::parse_pacman_info(&info_stdout);

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
                    source: AppSource::Pacman,
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
        let provider = PacmanProvider::new();
        assert_eq!(provider.name(), "pacman");
    }

    #[test]
    fn test_parse_pacman_info_valid() {
        let output = "\
Name            : git
Description     : the fast distributed version control system
Name            : curl
Description     : command line tool for transferring data
";
        let packages = PacmanProvider::parse_pacman_info(output);
        assert_eq!(packages.len(), 2);
        assert_eq!(packages[0].0, "git");
        assert_eq!(
            packages[0].1,
            Some("the fast distributed version control system".to_string())
        );
        assert_eq!(packages[1].0, "curl");
    }

    #[test]
    fn test_parse_pacman_info_empty() {
        let packages = PacmanProvider::parse_pacman_info("");
        assert!(packages.is_empty());
    }

    #[test]
    fn test_parse_pacman_info_single() {
        let output = "Name            : vim\nDescription     : Vi Improved\n";
        let packages = PacmanProvider::parse_pacman_info(output);
        assert_eq!(packages.len(), 1);
        assert_eq!(packages[0].0, "vim");
        assert_eq!(packages[0].1, Some("Vi Improved".to_string()));
    }

    #[test]
    fn test_parse_pacman_info_no_description() {
        let output = "Name            : somepkg\n";
        let packages = PacmanProvider::parse_pacman_info(output);
        assert_eq!(packages.len(), 1);
        assert_eq!(packages[0].0, "somepkg");
        assert_eq!(packages[0].1, None);
    }
}
