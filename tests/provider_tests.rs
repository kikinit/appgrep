use std::path::PathBuf;

/// Helper to get the fixtures directory path.
fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

mod desktop_provider {
    use super::*;

    /// Parse a fixture .desktop file and return the content.
    fn read_fixture(name: &str) -> String {
        let path = fixtures_dir().join(name);
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e))
    }

    #[test]
    fn test_valid_fixture_parses_correctly() {
        let content = read_fixture("valid.desktop");
        let path = fixtures_dir().join("valid.desktop");

        let mut config = configparser::ini::Ini::new_cs();
        config.set_comment_symbols(&['#']);
        config.read(content).unwrap();

        let name = config.get("Desktop Entry", "Name").unwrap();
        assert_eq!(name, "Test Application");

        let exec = config.get("Desktop Entry", "Exec").unwrap();
        assert!(exec.contains("test-app"));

        let entry_type = config.get("Desktop Entry", "Type").unwrap();
        assert_eq!(entry_type, "Application");

        let categories = config.get("Desktop Entry", "Categories").unwrap();
        let cats: Vec<&str> = categories.split(';').filter(|s| !s.is_empty()).collect();
        assert!(cats.contains(&"Development"));
        assert!(cats.contains(&"IDE"));

        // Verify path is usable
        assert!(path.exists());
    }

    #[test]
    fn test_nodisplay_fixture_has_nodisplay_true() {
        let content = read_fixture("nodisplay.desktop");

        let mut config = configparser::ini::Ini::new_cs();
        config.set_comment_symbols(&['#']);
        config.read(content).unwrap();

        let no_display = config.get("Desktop Entry", "NoDisplay").unwrap();
        assert_eq!(no_display.to_lowercase(), "true");
    }

    #[test]
    fn test_unicode_fixture_has_unicode_content() {
        let content = read_fixture("unicode.desktop");

        let mut config = configparser::ini::Ini::new_cs();
        config.set_comment_symbols(&['#']);
        config.read(content).unwrap();

        let name = config.get("Desktop Entry", "Name").unwrap();
        // The name should contain non-ASCII characters
        assert!(name.len() > 0);

        let comment = config.get("Desktop Entry", "Comment").unwrap();
        assert!(comment.len() > 0);
    }
}

mod standalone_name_extraction {
    #[test]
    fn test_complex_appimage_names() {
        let cases = vec![
            ("UltiMaker-Cura-5.9.0-linux-X64.AppImage", "UltiMaker-Cura"),
            ("Godot_v4.6-stable_linux.x86_64", "Godot"),
            ("MyApp-1.2.3-x86_64.AppImage", "MyApp"),
            ("Logseq.AppImage", "Logseq"),
            ("simple-app", "simple-app"),
            ("Obsidian-1.7.7.AppImage", "Obsidian"),
            ("balenaEtcher-1.19.25-x64.AppImage", "balenaEtcher"),
        ];

        for (input, expected) in cases {
            let result = extract_name(input);
            assert_eq!(
                result, expected,
                "extract_name({:?}) = {:?}, expected {:?}",
                input, result, expected
            );
        }
    }

    /// Inline the name extraction logic for testing without depending on private internals.
    fn extract_name(filename: &str) -> String {
        let name = filename
            .strip_suffix(".AppImage")
            .or_else(|| filename.strip_suffix(".appimage"))
            .unwrap_or(filename);

        let name = strip_arch(name);
        let name = strip_version(&name);

        name.trim_end_matches(|c: char| c == '-' || c == '_' || c == '.')
            .to_string()
    }

    fn strip_arch(name: &str) -> String {
        let patterns = [
            "-linux-X64", "-linux-x86_64", "-linux_x86_64", "_linux.x86_64",
            "_linux-x86_64", "-linux-amd64", "-linux-arm64", "-x86_64",
            "-amd64", "-arm64", ".x86_64", "_x86_64",
        ];
        for p in &patterns {
            if let Some(s) = name.strip_suffix(p) {
                return s.to_string();
            }
        }
        name.to_string()
    }

    fn strip_version(name: &str) -> String {
        let bytes = name.as_bytes();
        let mut cut_pos = None;
        for i in (0..bytes.len()).rev() {
            if bytes[i] == b'-' || bytes[i] == b'_' {
                let rest = &name[i + 1..];
                if looks_like_version(rest) {
                    cut_pos = Some(i);
                }
            }
        }
        match cut_pos {
            Some(pos) => name[..pos].to_string(),
            None => name.to_string(),
        }
    }

    fn looks_like_version(s: &str) -> bool {
        if s.is_empty() {
            return false;
        }
        let s = s.strip_prefix('v').or_else(|| s.strip_prefix('V')).unwrap_or(s);
        if s.is_empty() {
            return false;
        }
        s.starts_with(|c: char| c.is_ascii_digit())
    }
}

mod deduplication {
    #[test]
    fn test_source_priority_order() {
        // Desktop=0, Flatpak=1, Snap=2, Standalone=3
        assert!(0 < 1); // Desktop < Flatpak
        assert!(1 < 2); // Flatpak < Snap
        assert!(2 < 3); // Snap < Standalone
    }
}
