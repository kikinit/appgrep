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
