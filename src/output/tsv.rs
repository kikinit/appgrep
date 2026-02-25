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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{AppSource, Application};

    fn make_app(name: &str) -> Application {
        Application {
            name: name.to_string(),
            exec_command: format!("/usr/bin/{}", name.to_lowercase()),
            source: AppSource::Desktop,
            location: String::new(),
            icon: None,
            categories: Vec::new(),
            description: Some(format!("{} app", name)),
        }
    }

    #[test]
    fn test_tsv_empty() {
        let apps: Vec<Application> = vec![];
        let mut buf = Vec::new();
        format_tsv(&apps, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        // Should still have header
        assert_eq!(output, "name\texec\tsource\tdescription\n");
    }

    #[test]
    fn test_tsv_single() {
        let apps = vec![make_app("Firefox")];
        let mut buf = Vec::new();
        format_tsv(&apps, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "name\texec\tsource\tdescription");
        assert!(lines[1].contains("Firefox"));
        assert!(lines[1].contains("\t"));
    }

    #[test]
    fn test_tsv_tab_separation() {
        let apps = vec![make_app("Firefox")];
        let mut buf = Vec::new();
        format_tsv(&apps, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        let data_line = output.lines().nth(1).unwrap();
        let fields: Vec<&str> = data_line.split('\t').collect();
        assert_eq!(fields.len(), 4);
        assert_eq!(fields[0], "Firefox");
        assert_eq!(fields[1], "/usr/bin/firefox");
        assert_eq!(fields[2], "desktop");
        assert_eq!(fields[3], "Firefox app");
    }

    #[test]
    fn test_tsv_multiple() {
        let apps = vec![make_app("Firefox"), make_app("GIMP"), make_app("VLC")];
        let mut buf = Vec::new();
        format_tsv(&apps, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 4); // header + 3 rows
    }
}
