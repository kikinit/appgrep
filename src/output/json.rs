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
            icon: Some(name.to_lowercase()),
            categories: vec!["Utility".to_string()],
            description: Some(format!("{} application", name)),
        }
    }

    #[test]
    fn test_json_list_empty() {
        let apps: Vec<Application> = vec![];
        let mut buf = Vec::new();
        format_json_list(&apps, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(output.trim()).unwrap();
        assert!(parsed.is_array());
        assert_eq!(parsed.as_array().unwrap().len(), 0);
    }

    #[test]
    fn test_json_list_single() {
        let apps = vec![make_app("Firefox")];
        let mut buf = Vec::new();
        format_json_list(&apps, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(output.trim()).unwrap();
        assert!(parsed.is_array());
        assert_eq!(parsed.as_array().unwrap().len(), 1);
        assert_eq!(parsed[0]["name"], "Firefox");
    }

    #[test]
    fn test_json_list_multiple() {
        let apps = vec![make_app("Firefox"), make_app("GIMP"), make_app("VLC")];
        let mut buf = Vec::new();
        format_json_list(&apps, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(output.trim()).unwrap();
        assert_eq!(parsed.as_array().unwrap().len(), 3);
    }

    #[test]
    fn test_json_list_correct_fields() {
        let apps = vec![make_app("Firefox")];
        let mut buf = Vec::new();
        format_json_list(&apps, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(output.trim()).unwrap();
        let app = &parsed[0];
        assert!(app.get("name").is_some());
        assert!(app.get("exec_command").is_some());
        assert!(app.get("source").is_some());
        assert!(app.get("location").is_some());
        assert!(app.get("icon").is_some());
        assert!(app.get("categories").is_some());
        assert!(app.get("description").is_some());
    }

    #[test]
    fn test_json_single() {
        let app = make_app("Firefox");
        let mut buf = Vec::new();
        format_json_single(&app, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(output.trim()).unwrap();
        assert_eq!(parsed["name"], "Firefox");
        assert_eq!(parsed["source"], "desktop");
    }
}
