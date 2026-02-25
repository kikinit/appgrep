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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{AppSource, Application};

    fn make_app(name: &str, exec: &str) -> Application {
        Application {
            name: name.to_string(),
            exec_command: exec.to_string(),
            source: AppSource::Desktop,
            location: String::new(),
            icon: None,
            categories: Vec::new(),
            description: None,
        }
    }

    #[test]
    fn test_exec_empty() {
        let apps: Vec<Application> = vec![];
        let mut buf = Vec::new();
        format_exec(&apps, &mut buf).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "");
    }

    #[test]
    fn test_exec_single() {
        let apps = vec![make_app("Firefox", "/usr/bin/firefox")];
        let mut buf = Vec::new();
        format_exec(&apps, &mut buf).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "/usr/bin/firefox\n");
    }

    #[test]
    fn test_exec_multiple() {
        let apps = vec![
            make_app("Firefox", "/usr/bin/firefox"),
            make_app("GIMP", "/usr/bin/gimp"),
        ];
        let mut buf = Vec::new();
        format_exec(&apps, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "/usr/bin/firefox");
        assert_eq!(lines[1], "/usr/bin/gimp");
    }
}
