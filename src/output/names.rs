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
            description: None,
        }
    }

    #[test]
    fn test_names_empty() {
        let apps: Vec<Application> = vec![];
        let mut buf = Vec::new();
        format_names(&apps, &mut buf).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "");
    }

    #[test]
    fn test_names_single() {
        let apps = vec![make_app("Firefox")];
        let mut buf = Vec::new();
        format_names(&apps, &mut buf).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "Firefox\n");
    }

    #[test]
    fn test_names_multiple() {
        let apps = vec![make_app("Firefox"), make_app("GIMP"), make_app("VLC")];
        let mut buf = Vec::new();
        format_names(&apps, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "Firefox");
        assert_eq!(lines[1], "GIMP");
        assert_eq!(lines[2], "VLC");
    }
}
