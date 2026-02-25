use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use crate::app::{AppSource, Application};
use crate::provider::{AppProvider, ProviderError};

pub struct StandaloneProvider;

impl StandaloneProvider {
    pub fn new() -> Self {
        Self
    }

    /// Directories to scan for standalone executables.
    fn scan_dirs() -> Vec<PathBuf> {
        let mut dirs = Vec::new();

        if let Some(home) = dirs::home_dir() {
            dirs.push(home.join("Applications"));
            dirs.push(home.join(".local/bin"));
            dirs.push(home.join("bin"));
        }

        dirs.push(PathBuf::from("/opt"));

        dirs
    }

    /// Check if a file is an executable (non-directory, has execute bit).
    fn is_executable(path: &Path) -> bool {
        if let Ok(metadata) = fs::metadata(path) {
            if metadata.is_file() {
                return metadata.permissions().mode() & 0o111 != 0;
            }
        }
        false
    }

    /// Check if a symlink points into /usr/bin (skip these).
    fn is_usr_bin_symlink(path: &Path) -> bool {
        if let Ok(target) = fs::read_link(path) {
            let resolved = if target.is_absolute() {
                target
            } else {
                path.parent().map(|p| p.join(&target)).unwrap_or(target)
            };
            if let Ok(canonical) = resolved.canonicalize() {
                return canonical.starts_with("/usr/bin");
            }
        }
        false
    }

    /// Extract a human-friendly name from a filename.
    pub fn extract_name(filename: &str) -> String {
        // Strip known extensions
        let name = filename
            .strip_suffix(".AppImage")
            .or_else(|| filename.strip_suffix(".appimage"))
            .unwrap_or(filename);

        // Strip architecture suffixes like linux.x86_64, linux-X64, x86_64, etc.
        let name = strip_arch_suffix(name);

        // Strip version patterns from the end: -1.2.3, _1.2.3, -v1.2.3
        let name = strip_version_suffix(&name);

        // Strip trailing separators
        name.trim_end_matches(|c: char| c == '-' || c == '_' || c == '.')
            .to_string()
    }

    fn scan_directory(dir: &Path, depth: usize) -> Vec<Application> {
        let mut apps = Vec::new();

        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return apps,
        };

        for entry in entries.flatten() {
            let path = entry.path();

            // For /opt, go one level deep
            if path.is_dir() && depth > 0 {
                apps.extend(Self::scan_directory(&path, 0));
                continue;
            }

            if !Self::is_executable(&path) {
                continue;
            }

            if Self::is_usr_bin_symlink(&path) {
                continue;
            }

            let filename = match path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };

            let name = Self::extract_name(&filename);
            if name.is_empty() {
                continue;
            }

            let abs_path = path.canonicalize().unwrap_or(path.clone());
            let location = abs_path.to_string_lossy().to_string();

            apps.push(Application {
                name,
                exec_command: location.clone(),
                source: AppSource::Standalone,
                location,
                icon: None,
                categories: Vec::new(),
                description: None,
            });
        }

        apps
    }
}

/// Strip architecture suffixes from a name.
fn strip_arch_suffix(name: &str) -> String {
    let patterns = [
        "-linux-X64",
        "-linux-x86_64",
        "-linux_x86_64",
        "_linux.x86_64",
        "_linux-x86_64",
        "-linux-amd64",
        "-linux-arm64",
        "-x86_64",
        "-amd64",
        "-arm64",
        ".x86_64",
        "_x86_64",
    ];

    for pattern in &patterns {
        if let Some(stripped) = name.strip_suffix(pattern) {
            return stripped.to_string();
        }
    }

    name.to_string()
}

/// Strip version patterns from the end of a name.
/// Matches: -1.2.3, _1.2.3, -v1.2.3, _v1.2.3, -v4.6-stable, etc.
fn strip_version_suffix(name: &str) -> String {
    let bytes = name.as_bytes();
    let mut cut_pos = None;

    for i in (0..bytes.len()).rev() {
        if bytes[i] == b'-' || bytes[i] == b'_' {
            let rest = &name[i + 1..];
            if looks_like_version(rest) {
                cut_pos = Some(i);
            }
            // Always continue scanning left — a version like v4.6-stable
            // has non-version segments after the initial version separator
        }
    }

    match cut_pos {
        Some(pos) => name[..pos].to_string(),
        None => name.to_string(),
    }
}

fn looks_like_version(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let s = s.strip_prefix('v').or_else(|| s.strip_prefix('V')).unwrap_or(s);
    if s.is_empty() {
        return false;
    }
    s.starts_with(|c: char| c.is_ascii_digit())
}

impl AppProvider for StandaloneProvider {
    fn name(&self) -> &str {
        "standalone"
    }

    fn is_available(&self) -> bool {
        true
    }

    fn discover(&self) -> Result<Vec<Application>, ProviderError> {
        let mut apps = Vec::new();

        for dir in Self::scan_dirs() {
            if !dir.is_dir() {
                continue;
            }

            let depth = if dir == PathBuf::from("/opt") { 1 } else { 0 };
            apps.extend(Self::scan_directory(&dir, depth));
        }

        Ok(apps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_name_appimage() {
        assert_eq!(
            StandaloneProvider::extract_name("UltiMaker-Cura-5.9.0-linux-X64.AppImage"),
            "UltiMaker-Cura"
        );
    }

    #[test]
    fn test_extract_name_versioned() {
        assert_eq!(
            StandaloneProvider::extract_name("Godot_v4.6-stable_linux.x86_64"),
            "Godot"
        );
    }

    #[test]
    fn test_extract_name_simple() {
        assert_eq!(StandaloneProvider::extract_name("myapp"), "myapp");
    }

    #[test]
    fn test_extract_name_version_number() {
        assert_eq!(
            StandaloneProvider::extract_name("MyApp-1.2.3-x86_64.AppImage"),
            "MyApp"
        );
    }

    #[test]
    fn test_extract_name_plain_appimage() {
        assert_eq!(
            StandaloneProvider::extract_name("Logseq.AppImage"),
            "Logseq"
        );
    }

    #[test]
    fn test_strip_version_suffix() {
        assert_eq!(strip_version_suffix("app-1.2.3"), "app");
        assert_eq!(strip_version_suffix("app_v2.0"), "app");
        assert_eq!(strip_version_suffix("app"), "app");
        assert_eq!(strip_version_suffix("My-App-1.0.0"), "My-App");
    }

    #[test]
    fn test_strip_arch_suffix() {
        assert_eq!(strip_arch_suffix("app-linux-X64"), "app");
        assert_eq!(strip_arch_suffix("app_linux.x86_64"), "app");
        assert_eq!(strip_arch_suffix("app-x86_64"), "app");
        assert_eq!(strip_arch_suffix("app"), "app");
    }
}
