use std::process::Command;

use crate::app::{AppSource, Application};
use crate::provider::{AppProvider, ProviderError};

pub struct FlatpakProvider;

impl FlatpakProvider {
    pub fn new() -> Self {
        Self
    }

    fn has_flatpak() -> bool {
        Command::new("which")
            .arg("flatpak")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    fn parse_flatpak_output(output: &str) -> Vec<Application> {
        let mut apps = Vec::new();

        for line in output.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() < 2 {
                continue;
            }

            let name = parts[0].trim().to_string();
            let app_id = parts[1].trim().to_string();
            let description = parts
                .get(2)
                .map(|d| d.trim().to_string())
                .filter(|d| !d.is_empty());

            if name.is_empty() || app_id.is_empty() {
                continue;
            }

            apps.push(Application {
                name,
                exec_command: format!("flatpak run {}", app_id),
                source: AppSource::Flatpak,
                location: app_id.clone(),
                icon: Some(app_id),
                categories: Vec::new(),
                description,
            });
        }

        apps
    }
}

impl AppProvider for FlatpakProvider {
    fn name(&self) -> &str {
        "flatpak"
    }

    fn is_available(&self) -> bool {
        Self::has_flatpak()
    }

    fn discover(&self) -> Result<Vec<Application>, ProviderError> {
        if !self.is_available() {
            return Ok(Vec::new());
        }

        let output = Command::new("flatpak")
            .args(["list", "--app", "--columns=name,application,description"])
            .output()
            .map_err(ProviderError::Io)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("appgrep: warning: flatpak list failed: {}", stderr.trim());
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(Self::parse_flatpak_output(&stdout))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_flatpak_output() {
        let output = "Firefox\torg.mozilla.firefox\tWeb Browser\nLibreOffice\torg.libreoffice.LibreOffice\tOffice Suite\n";
        let apps = FlatpakProvider::parse_flatpak_output(output);
        assert_eq!(apps.len(), 2);
        assert_eq!(apps[0].name, "Firefox");
        assert_eq!(apps[0].exec_command, "flatpak run org.mozilla.firefox");
        assert_eq!(apps[0].location, "org.mozilla.firefox");
        assert_eq!(apps[0].description, Some("Web Browser".to_string()));
        assert_eq!(apps[1].name, "LibreOffice");
    }

    #[test]
    fn test_parse_empty_output() {
        let apps = FlatpakProvider::parse_flatpak_output("");
        assert!(apps.is_empty());
    }

    #[test]
    fn test_parse_malformed_line() {
        let apps = FlatpakProvider::parse_flatpak_output("OnlyOnePart\n");
        assert!(apps.is_empty());
    }

    #[test]
    fn test_parse_no_description() {
        let apps = FlatpakProvider::parse_flatpak_output("MyApp\tcom.example.myapp\t\n");
        assert_eq!(apps.len(), 1);
        assert_eq!(apps[0].name, "MyApp");
        assert!(apps[0].description.is_none());
    }
}
