use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use configparser::ini::Ini;

use crate::app::{AppSource, Application};
use crate::provider::{AppProvider, ProviderError};

pub struct DesktopProvider;

impl DesktopProvider {
    pub fn new() -> Self {
        Self
    }

    /// Collect all XDG application directories.
    fn app_dirs() -> Vec<PathBuf> {
        let mut dirs = Vec::new();

        // $XDG_DATA_HOME/applications/ (default ~/.local/share/applications/)
        if let Some(data_home) = std::env::var_os("XDG_DATA_HOME") {
            dirs.push(PathBuf::from(data_home).join("applications"));
        } else if let Some(home) = dirs::home_dir() {
            dirs.push(home.join(".local/share/applications"));
        }

        // Standard system directories
        dirs.push(PathBuf::from("/usr/share/applications"));
        dirs.push(PathBuf::from("/usr/local/share/applications"));

        // $XDG_DATA_DIRS entries
        if let Ok(data_dirs) = std::env::var("XDG_DATA_DIRS") {
            for dir in data_dirs.split(':') {
                if !dir.is_empty() {
                    let app_dir = PathBuf::from(dir).join("applications");
                    if !dirs.contains(&app_dir) {
                        dirs.push(app_dir);
                    }
                }
            }
        }

        dirs
    }

    /// Parse a single .desktop file into an Application.
    fn parse_desktop_file(path: &Path) -> Result<Option<Application>, ProviderError> {
        let content = fs::read_to_string(path).map_err(ProviderError::Io)?;
        Self::parse_desktop_content(&content, path)
    }

    /// Parse desktop entry content string into an Application.
    pub fn parse_desktop_content(
        content: &str,
        path: &Path,
    ) -> Result<Option<Application>, ProviderError> {
        let mut config = Ini::new_cs();
        config.set_comment_symbols(&['#']);
        config
            .read(content.to_string())
            .map_err(|e| ProviderError::ParseError(format!("{}: {}", path.display(), e)))?;

        let section = "Desktop Entry";

        // Check Type
        let entry_type = config.get(section, "Type").unwrap_or_default();
        if entry_type.to_lowercase() != "application" {
            return Ok(None);
        }

        // Check NoDisplay and Hidden
        let no_display = config.get(section, "NoDisplay").unwrap_or_default();
        if no_display.to_lowercase() == "true" {
            return Ok(None);
        }

        let hidden = config.get(section, "Hidden").unwrap_or_default();
        if hidden.to_lowercase() == "true" {
            return Ok(None);
        }

        // Name is required
        let name = match config.get(section, "Name") {
            Some(n) if !n.is_empty() => n,
            _ => return Ok(None),
        };

        // Exec is required
        let exec_raw = match config.get(section, "Exec") {
            Some(e) if !e.is_empty() => e,
            _ => return Ok(None),
        };

        let exec_command = strip_field_codes(&exec_raw);

        let icon = config.get(section, "Icon").filter(|s| !s.is_empty());

        let categories = config
            .get(section, "Categories")
            .map(|c| {
                c.split(';')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default();

        let description = config.get(section, "Comment").filter(|s| !s.is_empty());

        Ok(Some(Application {
            name,
            exec_command,
            source: AppSource::Desktop,
            location: path.to_string_lossy().to_string(),
            icon,
            categories,
            description,
        }))
    }
}

/// Strip XDG field codes from an Exec string.
pub fn strip_field_codes(exec: &str) -> String {
    let codes = [
        "%f", "%F", "%u", "%U", "%d", "%D", "%n", "%N", "%i", "%c", "%k", "%v", "%m",
    ];
    let mut result = exec.to_string();
    for code in &codes {
        result = result.replace(code, "");
    }
    // Clean up extra whitespace
    result.split_whitespace().collect::<Vec<_>>().join(" ")
}

impl AppProvider for DesktopProvider {
    fn name(&self) -> &str {
        "desktop"
    }

    fn is_available(&self) -> bool {
        true
    }

    fn discover(&self) -> Result<Vec<Application>, ProviderError> {
        let mut apps = Vec::new();
        let mut seen_paths = HashSet::new();

        for dir in Self::app_dirs() {
            if !dir.is_dir() {
                continue;
            }

            let entries = match fs::read_dir(&dir) {
                Ok(entries) => entries,
                Err(e) => {
                    eprintln!("appgrep: warning: cannot read {}: {}", dir.display(), e);
                    continue;
                }
            };

            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("desktop") {
                    continue;
                }

                // Skip duplicates by canonical path
                let canonical = path.canonicalize().unwrap_or_else(|_| path.clone());
                if !seen_paths.insert(canonical) {
                    continue;
                }

                match Self::parse_desktop_file(&path) {
                    Ok(Some(app)) => apps.push(app),
                    Ok(None) => {}
                    Err(e) => {
                        eprintln!("appgrep: warning: {}: {}", path.display(), e);
                    }
                }
            }
        }

        Ok(apps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_parse_valid_desktop_entry() {
        let content = r#"[Desktop Entry]
Type=Application
Name=Firefox
Exec=/usr/bin/firefox %u
Icon=firefox
Categories=Network;WebBrowser;
Comment=Web Browser
"#;
        let path = PathBuf::from("/usr/share/applications/firefox.desktop");
        let result = DesktopProvider::parse_desktop_content(content, &path)
            .unwrap()
            .unwrap();
        assert_eq!(result.name, "Firefox");
        assert_eq!(result.exec_command, "/usr/bin/firefox");
        assert_eq!(result.source, AppSource::Desktop);
        assert_eq!(result.icon, Some("firefox".to_string()));
        assert_eq!(result.categories, vec!["Network", "WebBrowser"]);
        assert_eq!(result.description, Some("Web Browser".to_string()));
    }

    #[test]
    fn test_skip_nodisplay() {
        let content = r#"[Desktop Entry]
Type=Application
Name=Hidden App
Exec=/usr/bin/hidden
NoDisplay=true
"#;
        let path = PathBuf::from("/test/hidden.desktop");
        let result = DesktopProvider::parse_desktop_content(content, &path).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_skip_hidden() {
        let content = r#"[Desktop Entry]
Type=Application
Name=Hidden App
Exec=/usr/bin/hidden
Hidden=true
"#;
        let path = PathBuf::from("/test/hidden.desktop");
        let result = DesktopProvider::parse_desktop_content(content, &path).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_skip_non_application_type() {
        let content = r#"[Desktop Entry]
Type=Link
Name=Some Link
Exec=/usr/bin/something
"#;
        let path = PathBuf::from("/test/link.desktop");
        let result = DesktopProvider::parse_desktop_content(content, &path).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_strip_field_codes() {
        assert_eq!(strip_field_codes("/usr/bin/app %u"), "/usr/bin/app");
        assert_eq!(strip_field_codes("/usr/bin/app %f %F"), "/usr/bin/app");
        assert_eq!(
            strip_field_codes("/usr/bin/app --flag %U --other"),
            "/usr/bin/app --flag --other"
        );
        assert_eq!(strip_field_codes("app"), "app");
    }

    #[test]
    fn test_unicode_name() {
        let content = "[Desktop Entry]\nType=Application\nName=Uc\u{0327}ode App \u{1f680}\nExec=/usr/bin/unicode-app\nComment=An app with unicode: emojis \u{2728}\n";
        let path = PathBuf::from("/test/unicode.desktop");
        let result = DesktopProvider::parse_desktop_content(content, &path)
            .unwrap()
            .unwrap();
        assert!(result.name.contains("ode App"));
        assert!(result.description.is_some());
    }

    #[test]
    fn test_missing_exec() {
        let content = r#"[Desktop Entry]
Type=Application
Name=No Exec App
"#;
        let path = PathBuf::from("/test/noexec.desktop");
        let result = DesktopProvider::parse_desktop_content(content, &path).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_missing_name() {
        let content = r#"[Desktop Entry]
Type=Application
Exec=/usr/bin/something
"#;
        let path = PathBuf::from("/test/noname.desktop");
        let result = DesktopProvider::parse_desktop_content(content, &path).unwrap();
        assert!(result.is_none());
    }
}
