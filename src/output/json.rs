use crate::app::Application;

pub fn format_json_list(
    apps: &[Application],
    w: &mut dyn std::io::Write,
) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(apps)?;
    writeln!(w, "{}", json)?;
    Ok(())
}

pub fn format_json_single(
    app: &Application,
    w: &mut dyn std::io::Write,
) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(app)?;
    writeln!(w, "{}", json)?;
    Ok(())
}
