use crate::app::Application;

pub fn format_names(
    apps: &[Application],
    w: &mut dyn std::io::Write,
) -> anyhow::Result<()> {
    for app in apps {
        writeln!(w, "{}", app.name)?;
    }
    Ok(())
}
