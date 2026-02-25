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
