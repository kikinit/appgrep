use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::app::{AppSource, Application};
use crate::provider::{AppProvider, ProviderError};

pub struct SnapProvider;

impl SnapProvider {
    pub fn new() -> Self {
        Self
    }

    fn has_snap() -> bool {
        Command::new("which")
            .arg("snap")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    fn parse_snap_list(output: &str) -> Vec<String> {
        let mut names = Vec::new();

        for (i, line) in output.lines().enumerate() {
            // Skip header
            if i == 0 {
                continue;
            }

            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            let name = parts[0].to_string();

            // Check if disabled
            if let Some(notes) = parts.last() {
                if notes.contains("disabled") {
                    continue;
                }
            }

            names.push(name);
        }

        names
    }

    /// Try to extract richer metadata from a snap's .desktop file.
    fn enrich_from_desktop(
        name: &str,
    ) -> (Option<String>, Option<String>, Vec<String>, Option<String>) {
        let gui_dir = PathBuf::from(format!("/snap/{}/current/meta/gui", name));
        let mut display_name = None;
        let mut icon = None;
        let mut categories = Vec::new();
        let mut description = None;

        if gui_dir.is_dir() {
            if let Ok(entries) = fs::read_dir(&gui_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|e| e.to_str()) == Some("desktop") {
                        if let Ok(content) = fs::read_to_string(&path) {
                            let mut config = configparser::ini::Ini::new_cs();
                            config.set_comment_symbols(&['#']);
                            if config.read(content).is_ok() {
                                let section = "Desktop Entry";
                                if let Some(n) = config.get(section, "Name") {
                                    if !n.is_empty() {
                                        display_name = Some(n);
                                    }
                                }
                                if let Some(i) = config.get(section, "Icon") {
                                    if !i.is_empty() {
                                        icon = Some(i);
                                    }
                                }
                                if let Some(c) = config.get(section, "Categories") {
                                    categories = c
                                        .split(';')
                                        .map(|s| s.trim().to_string())
                                        .filter(|s| !s.is_empty())
                                        .collect();
                                }
                                if let Some(d) = config.get(section, "Comment") {
                                    if !d.is_empty() {
                                        description = Some(d);
                                    }
                                }
                            }
                        }
                        break; // Use the first .desktop file
                    }
                }
            }
        }

        (display_name, icon, categories, description)
    }
}

impl AppProvider for SnapProvider {
    fn name(&self) -> &str {
        "snap"
    }

    fn is_available(&self) -> bool {
        Self::has_snap()
    }

    fn discover(&self) -> Result<Vec<Application>, ProviderError> {
        if !self.is_available() {
            return Ok(Vec::new());
        }

        let output = Command::new("snap")
            .arg("list")
            .output()
            .map_err(ProviderError::Io)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("appgrep: warning: snap list failed: {}", stderr.trim());
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let snap_names = Self::parse_snap_list(&stdout);

        let mut apps = Vec::new();
        for name in snap_names {
            let (display_name, icon, categories, description) = Self::enrich_from_desktop(&name);

            apps.push(Application {
                name: display_name.unwrap_or_else(|| name.clone()),
                exec_command: format!("snap run {}", name),
                source: AppSource::Snap,
                location: name,
                icon,
                categories,
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
    fn test_parse_snap_list() {
        let output = "Name      Version    Rev    Tracking       Publisher   Notes\nfirefox   128.0      4173   latest/stable  mozilla     -\ncore22    20240111   1380   latest/stable  canonical   base\nspotify   1.2.26     73    latest/stable  spotify     -\n";
        let names = SnapProvider::parse_snap_list(output);
        assert_eq!(names, vec!["firefox", "core22", "spotify"]);
    }

    #[test]
    fn test_parse_snap_list_skips_disabled() {
        let output = "Name      Version    Rev    Tracking       Publisher   Notes\nmyapp     1.0        10     latest/stable  me          disabled\nother     2.0        20     latest/stable  me          -\n";
        let names = SnapProvider::parse_snap_list(output);
        assert_eq!(names, vec!["other"]);
    }

    #[test]
    fn test_parse_empty_snap_list() {
        let output = "Name      Version    Rev    Tracking       Publisher   Notes\n";
        let names = SnapProvider::parse_snap_list(output);
        assert!(names.is_empty());
    }
}
