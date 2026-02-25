use crate::app::Application;

pub fn format_tsv(
    apps: &[Application],
    w: &mut dyn std::io::Write,
) -> anyhow::Result<()> {
    writeln!(w, "name\texec\tsource\tdescription")?;
    for app in apps {
        writeln!(
            w,
            "{}\t{}\t{}\t{}",
            app.name,
            app.exec_command,
            app.source,
            app.description.as_deref().unwrap_or("")
        )?;
    }
    Ok(())
}
