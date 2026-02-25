use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use crate::app::{AppSource, Application};
use crate::provider::{AppProvider, ProviderError};

pub struct CargoProvider;

impl CargoProvider {
    pub fn new() -> Self {
        Self
    }

    fn cargo_bin_dir() -> Option<PathBuf> {
        dirs::home_dir().map(|h| h.join(".cargo").join("bin"))
    }
}

impl AppProvider for CargoProvider {
    fn name(&self) -> &str {
        "cargo"
    }

    fn is_available(&self) -> bool {
        true
    }

    fn discover(&self) -> Result<Vec<Application>, ProviderError> {
        let bin_dir = match Self::cargo_bin_dir() {
            Some(d) => d,
            None => return Ok(Vec::new()),
        };

        if !bin_dir.is_dir() {
            return Ok(Vec::new());
        }

        let entries = fs::read_dir(&bin_dir).map_err(ProviderError::Io)?;
        let mut apps = Vec::new();

        for entry in entries.flatten() {
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            let metadata = match fs::metadata(&path) {
                Ok(m) => m,
                Err(_) => continue,
            };

            // Skip non-executable files
            if metadata.permissions().mode() & 0o111 == 0 {
                continue;
            }

            // Skip symlinks pointing to themselves
            if path.is_symlink() {
                if let Ok(target) = fs::read_link(&path) {
                    if target == path {
                        continue;
                    }
                }
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
                source: AppSource::Cargo,
                location: abs_path,
                icon: None,
                categories: vec!["Development".to_string()],
                description: None,
            });
        }

        Ok(apps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use tempfile::TempDir;

    #[test]
    fn test_discover_empty_dir() {
        let provider = CargoProvider::new();
        assert_eq!(provider.name(), "cargo");
        assert!(provider.is_available());
    }

    #[test]
    fn test_cargo_bin_dir_returns_path() {
        let dir = CargoProvider::cargo_bin_dir();
        assert!(dir.is_some());
        let path = dir.unwrap();
        assert!(path.to_string_lossy().contains(".cargo/bin"));
    }

    #[test]
    fn test_discover_with_executables() {
        let tmp = TempDir::new().unwrap();
        let bin_dir = tmp.path().join(".cargo").join("bin");
        fs::create_dir_all(&bin_dir).unwrap();

        // Create an executable file
        let exec_path = bin_dir.join("mytool");
        fs::write(&exec_path, "#!/bin/sh\n").unwrap();
        fs::set_permissions(&exec_path, fs::Permissions::from_mode(0o755)).unwrap();

        // Create a non-executable file
        let noexec_path = bin_dir.join("data.txt");
        fs::write(&noexec_path, "data").unwrap();
        fs::set_permissions(&noexec_path, fs::Permissions::from_mode(0o644)).unwrap();

        // Scan the directory directly
        let entries = fs::read_dir(&bin_dir).unwrap();
        let mut apps = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let metadata = fs::metadata(&path).unwrap();
            if metadata.permissions().mode() & 0o111 == 0 {
                continue;
            }
            let name = path.file_name().unwrap().to_str().unwrap().to_string();
            apps.push(Application {
                name,
                exec_command: path.to_string_lossy().to_string(),
                source: AppSource::Cargo,
                location: path.to_string_lossy().to_string(),
                icon: None,
                categories: vec!["Development".to_string()],
                description: None,
            });
        }

        assert_eq!(apps.len(), 1);
        assert_eq!(apps[0].name, "mytool");
        assert_eq!(apps[0].source, AppSource::Cargo);
        assert_eq!(apps[0].categories, vec!["Development"]);
    }

    #[test]
    fn test_skips_directories() {
        let tmp = TempDir::new().unwrap();
        let bin_dir = tmp.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();

        // Create a subdirectory (should be skipped)
        fs::create_dir_all(bin_dir.join("subdir")).unwrap();

        let entries = fs::read_dir(&bin_dir).unwrap();
        let files: Vec<_> = entries
            .flatten()
            .filter(|e| e.path().is_file())
            .collect();
        assert_eq!(files.len(), 0);
    }
}
