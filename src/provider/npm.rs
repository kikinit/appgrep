use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;

use crate::app::{AppSource, Application};
use crate::provider::{AppProvider, ProviderError};

pub struct NpmProvider;

impl NpmProvider {
    pub fn new() -> Self {
        Self
    }

    fn has_npm() -> bool {
        Command::new("which")
            .arg("npm")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    fn get_global_bin_dir() -> Option<PathBuf> {
        // Try npm root -g and derive bin dir
        if let Ok(output) = Command::new("npm").args(["root", "-g"]).output() {
            if output.status.success() {
                let root = String::from_utf8_lossy(&output.stdout).trim().to_string();
                // Replace trailing /node_modules with /bin
                if let Some(prefix) = root.strip_suffix("/node_modules") {
                    let bin_dir = PathBuf::from(format!("{}/bin", prefix));
                    if bin_dir.is_dir() {
                        return Some(bin_dir);
                    }
                }
                // Also try parent/bin
                let parent_bin = PathBuf::from(&root)
                    .parent()
                    .map(|p| p.join("bin"));
                if let Some(ref pb) = parent_bin {
                    if pb.is_dir() {
                        return parent_bin;
                    }
                }
            }
        }

        // Fallback paths
        if let Some(home) = dirs::home_dir() {
            let fallbacks = [
                home.join(".npm-global").join("bin"),
                home.join(".local")
                    .join("lib")
                    .join("node_modules")
                    .join(".bin"),
            ];
            for fb in &fallbacks {
                if fb.is_dir() {
                    return Some(fb.clone());
                }
            }
        }

        None
    }

    fn scan_bin_dir(bin_dir: &PathBuf) -> Vec<Application> {
        let entries = match fs::read_dir(bin_dir) {
            Ok(e) => e,
            Err(_) => return Vec::new(),
        };

        let mut apps = Vec::new();

        for entry in entries.flatten() {
            let path = entry.path();

            if !path.is_file() && !path.is_symlink() {
                continue;
            }

            // For symlinks, check the target exists
            if path.is_symlink() {
                match fs::metadata(&path) {
                    Ok(_) => {}
                    Err(_) => continue, // broken symlink
                }
            }

            let metadata = match fs::metadata(&path) {
                Ok(m) => m,
                Err(_) => continue,
            };

            // Skip non-executable files
            if metadata.permissions().mode() & 0o111 == 0 {
                continue;
            }

            let name = match path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };

            let abs_path = match path.canonicalize() {
                Ok(p) => p.to_string_lossy().to_string(),
                Err(_) => path.to_string_lossy().to_string(),
            };

            apps.push(Application {
                name,
                exec_command: abs_path.clone(),
                source: AppSource::Npm,
                location: abs_path,
                icon: None,
                categories: vec!["Development".to_string()],
                description: None,
            });
        }

        apps
    }
}

impl AppProvider for NpmProvider {
    fn name(&self) -> &str {
        "npm"
    }

    fn is_available(&self) -> bool {
        Self::has_npm()
    }

    fn discover(&self) -> Result<Vec<Application>, ProviderError> {
        let bin_dir = match Self::get_global_bin_dir() {
            Some(d) => d,
            None => return Ok(Vec::new()),
        };

        Ok(Self::scan_bin_dir(&bin_dir))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use tempfile::TempDir;

    #[test]
    fn test_provider_name() {
        let provider = NpmProvider::new();
        assert_eq!(provider.name(), "npm");
    }

    #[test]
    fn test_scan_empty_dir() {
        let tmp = TempDir::new().unwrap();
        let apps = NpmProvider::scan_bin_dir(&tmp.path().to_path_buf());
        assert!(apps.is_empty());
    }

    #[test]
    fn test_scan_with_executables() {
        let tmp = TempDir::new().unwrap();
        let bin_dir = tmp.path().to_path_buf();

        let exec_path = bin_dir.join("prettier");
        fs::write(&exec_path, "#!/usr/bin/env node\n").unwrap();
        fs::set_permissions(&exec_path, fs::Permissions::from_mode(0o755)).unwrap();

        let noexec_path = bin_dir.join("readme.txt");
        fs::write(&noexec_path, "info").unwrap();
        fs::set_permissions(&noexec_path, fs::Permissions::from_mode(0o644)).unwrap();

        let apps = NpmProvider::scan_bin_dir(&bin_dir);
        assert_eq!(apps.len(), 1);
        assert_eq!(apps[0].name, "prettier");
        assert_eq!(apps[0].source, AppSource::Npm);
        assert_eq!(apps[0].categories, vec!["Development"]);
    }

    #[test]
    fn test_scan_nonexistent_dir() {
        let apps = NpmProvider::scan_bin_dir(&PathBuf::from("/nonexistent/path/bin"));
        assert!(apps.is_empty());
    }
}
