use comfy_table::{Cell, ContentArrangement, Table};
use owo_colors::OwoColorize;

use crate::app::{AppSource, Application};

pub fn format_table(
    apps: &[Application],
    w: &mut dyn std::io::Write,
    no_color: bool,
) -> anyhow::Result<()> {
    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(vec!["Name", "Exec", "Source", "Description"]);

    for app in apps {
        let source_str = app.source.to_string();
        let source_display = if no_color {
            source_str
        } else {
            match app.source {
                AppSource::Desktop => source_str.green().to_string(),
                AppSource::Flatpak => source_str.blue().to_string(),
                AppSource::Snap => source_str.yellow().to_string(),
                AppSource::Standalone => source_str.cyan().to_string(),
                AppSource::Cargo => source_str.magenta().to_string(),
                AppSource::Npm => source_str.red().to_string(),
                AppSource::Dpkg => source_str.white().to_string(),
                AppSource::Rpm => source_str.bright_red().to_string(),
                AppSource::Pacman => source_str.bright_cyan().to_string(),
                AppSource::Brew => source_str.bright_yellow().to_string(),
            }
        };

        table.add_row(vec![
            Cell::new(&app.name),
            Cell::new(&app.exec_command),
            Cell::new(source_display),
            Cell::new(app.description.as_deref().unwrap_or("")),
        ]);
    }

    writeln!(w, "{}", table)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_app(name: &str, source: AppSource) -> Application {
        Application {
            name: name.to_string(),
            exec_command: format!("/usr/bin/{}", name.to_lowercase()),
            source,
            location: String::new(),
            icon: None,
            categories: Vec::new(),
            description: Some(format!("{} app", name)),
        }
    }

    #[test]
    fn test_table_empty() {
        let apps: Vec<Application> = vec![];
        let mut buf = Vec::new();
        format_table(&apps, &mut buf, true).unwrap();
        let output = String::from_utf8(buf).unwrap();
        // Should still have header
        assert!(output.contains("Name"));
        assert!(output.contains("Exec"));
    }

    #[test]
    fn test_table_contains_app_name() {
        let apps = vec![make_app("Firefox", AppSource::Desktop)];
        let mut buf = Vec::new();
        format_table(&apps, &mut buf, true).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Firefox"));
    }

    #[test]
    fn test_table_no_color() {
        let apps = vec![make_app("Firefox", AppSource::Desktop)];
        let mut buf = Vec::new();
        format_table(&apps, &mut buf, true).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("desktop"));
        // No ANSI escape codes when no_color is true
        assert!(!output.contains("\x1b["));
    }

    #[test]
    fn test_table_with_color() {
        let apps = vec![make_app("Firefox", AppSource::Desktop)];
        let mut buf = Vec::new();
        format_table(&apps, &mut buf, false).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Firefox"));
    }

    #[test]
    fn test_table_multiple_sources() {
        let apps = vec![
            make_app("Firefox", AppSource::Desktop),
            make_app("mytool", AppSource::Cargo),
            make_app("curl", AppSource::Dpkg),
        ];
        let mut buf = Vec::new();
        format_table(&apps, &mut buf, true).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Firefox"));
        assert!(output.contains("mytool"));
        assert!(output.contains("curl"));
    }
}
