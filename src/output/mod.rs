pub mod exec;
pub mod json;
pub mod names;
pub mod table;
pub mod tsv;

use crate::app::Application;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Table,
    Json,
    Tsv,
    Names,
    Exec,
}

pub struct Formatter {
    format: OutputFormat,
    no_color: bool,
}

impl Formatter {
    pub fn new(format: OutputFormat, no_color: bool) -> Self {
        Self { format, no_color }
    }

    pub fn format_list(
        &self,
        apps: &[Application],
        w: &mut dyn std::io::Write,
    ) -> anyhow::Result<()> {
        match self.format {
            OutputFormat::Table => table::format_table(apps, w, self.no_color),
            OutputFormat::Json => json::format_json_list(apps, w),
            OutputFormat::Tsv => tsv::format_tsv(apps, w),
            OutputFormat::Names => names::format_names(apps, w),
            OutputFormat::Exec => exec::format_exec(apps, w),
        }
    }

    pub fn format_info(
        &self,
        app: &Application,
        w: &mut dyn std::io::Write,
    ) -> anyhow::Result<()> {
        match self.format {
            OutputFormat::Json => json::format_json_single(app, w),
            _ => {
                writeln!(w, "Name:        {}", app.name)?;
                writeln!(w, "Exec:        {}", app.exec_command)?;
                writeln!(w, "Source:      {}", app.source)?;
                writeln!(w, "Location:    {}", app.location)?;
                writeln!(
                    w,
                    "Icon:        {}",
                    app.icon.as_deref().unwrap_or("-")
                )?;
                writeln!(
                    w,
                    "Categories:  {}",
                    if app.categories.is_empty() {
                        "-".to_string()
                    } else {
                        app.categories.join(", ")
                    }
                )?;
                writeln!(
                    w,
                    "Description: {}",
                    app.description.as_deref().unwrap_or("-")
                )?;
                Ok(())
            }
        }
    }

    pub fn format_has(
        &self,
        app: &Application,
        found: bool,
        w: &mut dyn std::io::Write,
    ) -> anyhow::Result<()> {
        if self.format == OutputFormat::Json {
            let obj = serde_json::json!({
                "found": found,
                "name": app.name,
                "source": app.source,
            });
            writeln!(w, "{}", serde_json::to_string_pretty(&obj)?)?;
        }
        // Silent for non-JSON formats (exit code only)
        Ok(())
    }

    pub fn format_has_not_found(
        &self,
        name: &str,
        w: &mut dyn std::io::Write,
    ) -> anyhow::Result<()> {
        if self.format == OutputFormat::Json {
            let obj = serde_json::json!({
                "found": false,
                "name": name,
            });
            writeln!(w, "{}", serde_json::to_string_pretty(&obj)?)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{AppSource, Application};

    fn make_app(name: &str) -> Application {
        Application {
            name: name.to_string(),
            exec_command: format!("/usr/bin/{}", name.to_lowercase()),
            source: AppSource::Desktop,
            location: format!("/usr/share/applications/{}.desktop", name.to_lowercase()),
            icon: Some("icon".to_string()),
            categories: vec!["Utility".to_string()],
            description: Some(format!("{} application", name)),
        }
    }

    fn make_minimal_app(name: &str) -> Application {
        Application {
            name: name.to_string(),
            exec_command: format!("/usr/bin/{}", name.to_lowercase()),
            source: AppSource::Standalone,
            location: String::new(),
            icon: None,
            categories: Vec::new(),
            description: None,
        }
    }

    #[test]
    fn test_format_info_plain() {
        let formatter = Formatter::new(OutputFormat::Table, true);
        let app = make_app("Firefox");
        let mut buf = Vec::new();
        formatter.format_info(&app, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Name:        Firefox"));
        assert!(output.contains("Exec:        /usr/bin/firefox"));
        assert!(output.contains("Source:      desktop"));
        assert!(output.contains("Icon:        icon"));
        assert!(output.contains("Categories:  Utility"));
        assert!(output.contains("Description: Firefox application"));
    }

    #[test]
    fn test_format_info_plain_minimal() {
        let formatter = Formatter::new(OutputFormat::Table, true);
        let app = make_minimal_app("mytool");
        let mut buf = Vec::new();
        formatter.format_info(&app, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Icon:        -"));
        assert!(output.contains("Categories:  -"));
        assert!(output.contains("Description: -"));
    }

    #[test]
    fn test_format_info_json() {
        let formatter = Formatter::new(OutputFormat::Json, false);
        let app = make_app("Firefox");
        let mut buf = Vec::new();
        formatter.format_info(&app, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(output.trim()).unwrap();
        assert_eq!(parsed["name"], "Firefox");
    }

    #[test]
    fn test_format_has_json() {
        let formatter = Formatter::new(OutputFormat::Json, false);
        let app = make_app("Firefox");
        let mut buf = Vec::new();
        formatter.format_has(&app, true, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(output.trim()).unwrap();
        assert_eq!(parsed["found"], true);
        assert_eq!(parsed["name"], "Firefox");
    }

    #[test]
    fn test_format_has_silent_non_json() {
        let formatter = Formatter::new(OutputFormat::Table, true);
        let app = make_app("Firefox");
        let mut buf = Vec::new();
        formatter.format_has(&app, true, &mut buf).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "");
    }

    #[test]
    fn test_format_has_not_found_json() {
        let formatter = Formatter::new(OutputFormat::Json, false);
        let mut buf = Vec::new();
        formatter.format_has_not_found("nonexistent", &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(output.trim()).unwrap();
        assert_eq!(parsed["found"], false);
        assert_eq!(parsed["name"], "nonexistent");
    }

    #[test]
    fn test_format_has_not_found_silent_non_json() {
        let formatter = Formatter::new(OutputFormat::Table, true);
        let mut buf = Vec::new();
        formatter.format_has_not_found("nonexistent", &mut buf).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "");
    }

    #[test]
    fn test_format_list_each_format() {
        let apps = vec![make_app("Firefox")];

        for format in [
            OutputFormat::Table,
            OutputFormat::Json,
            OutputFormat::Tsv,
            OutputFormat::Names,
            OutputFormat::Exec,
        ] {
            let formatter = Formatter::new(format, true);
            let mut buf = Vec::new();
            formatter.format_list(&apps, &mut buf).unwrap();
            let output = String::from_utf8(buf).unwrap();
            assert!(!output.is_empty(), "Format {:?} produced empty output", format);
        }
    }
}
