use crate::app::Application;

pub fn format_exec(
    apps: &[Application],
    w: &mut dyn std::io::Write,
) -> anyhow::Result<()> {
    for app in apps {
        writeln!(w, "{}", app.exec_command)?;
    }
    Ok(())
}
