use std::collections::HashMap;

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use rayon::prelude::*;

use crate::app::{AppSource, Application};
use crate::provider::desktop::DesktopProvider;
use crate::provider::flatpak::FlatpakProvider;
use crate::provider::snap::SnapProvider;
use crate::provider::standalone::StandaloneProvider;
use crate::provider::AppProvider;

pub struct DiscoveryEngine {
    providers: Vec<Box<dyn AppProvider>>,
}

impl DiscoveryEngine {
    pub fn new() -> Self {
        let providers: Vec<Box<dyn AppProvider>> = vec![
            Box::new(DesktopProvider::new()),
            Box::new(FlatpakProvider::new()),
            Box::new(SnapProvider::new()),
            Box::new(StandaloneProvider::new()),
        ];
        Self { providers }
    }

    /// Discover all applications from all available providers in parallel.
    pub fn discover_all(&self) -> Vec<Application> {
        let results: Vec<Vec<Application>> = self
            .providers
            .par_iter()
            .filter(|p| p.is_available())
            .map(|p| match p.discover() {
                Ok(apps) => apps,
                Err(e) => {
                    eprintln!("appgrep: warning: provider '{}' failed: {}", p.name(), e);
                    Vec::new()
                }
            })
            .collect();

        let all_apps: Vec<Application> = results.into_iter().flatten().collect();
        let mut deduped = Self::deduplicate(all_apps);
        deduped.sort();
        deduped
    }

    /// Discover applications filtered by source types.
    pub fn discover_filtered(&self, sources: &[AppSource]) -> Vec<Application> {
        let all = self.discover_all();
        all.into_iter()
            .filter(|app| sources.contains(&app.source))
            .collect()
    }

    /// Fuzzy search applications by name and description.
    pub fn search(&self, query: &str, apps: &[Application]) -> Vec<Application> {
        let matcher = SkimMatcherV2::default();
        let mut scored: Vec<(i64, &Application)> = apps
            .iter()
            .filter_map(|app| {
                let name_score = matcher.fuzzy_match(&app.name, query).unwrap_or(0);
                let desc_score = app
                    .description
                    .as_ref()
                    .and_then(|d| matcher.fuzzy_match(d, query))
                    .unwrap_or(0);
                let score = name_score.max(desc_score);
                if score > 0 {
                    Some((score, app))
                } else {
                    None
                }
            })
            .collect();

        scored.sort_by(|a, b| b.0.cmp(&a.0));
        scored.into_iter().map(|(_, app)| app.clone()).collect()
    }

    /// Find an application by name: exact case-insensitive match first, then fuzzy best.
    pub fn find_by_name(&self, name: &str, apps: &[Application]) -> Option<Application> {
        let lower = name.to_lowercase();

        // Exact case-insensitive match
        if let Some(app) = apps.iter().find(|a| a.name.to_lowercase() == lower) {
            return Some(app.clone());
        }

        // Fuzzy best match
        let matcher = SkimMatcherV2::default();
        let mut best: Option<(i64, &Application)> = None;
        for app in apps {
            if let Some(score) = matcher.fuzzy_match(&app.name, name) {
                match &best {
                    Some((best_score, _)) if score <= *best_score => {}
                    _ => best = Some((score, app)),
                }
            }
        }

        best.map(|(_, app)| app.clone())
    }

    /// Deduplicate applications by normalized exec command.
    /// When duplicates exist: prefer higher-priority source, then more metadata.
    fn deduplicate(apps: Vec<Application>) -> Vec<Application> {
        let mut groups: HashMap<String, Vec<Application>> = HashMap::new();

        for app in apps {
            let key = normalize_exec(&app.exec_command);
            groups.entry(key).or_default().push(app);
        }

        groups
            .into_values()
            .map(|mut group| {
                group.sort_by(|a, b| {
                    a.source
                        .priority()
                        .cmp(&b.source.priority())
                        .then_with(|| b.metadata_richness().cmp(&a.metadata_richness()))
                });
                group.into_iter().next().unwrap()
            })
            .collect()
    }
}

/// Normalize an exec command for deduplication comparison.
fn normalize_exec(exec: &str) -> String {
    let trimmed = exec.trim();
    // Strip quotes around the path
    let unquoted = trimmed
        .strip_prefix('"')
        .and_then(|s| s.find('"').map(|pos| &s[..pos]))
        .unwrap_or_else(|| {
            // No quotes: take the first whitespace-delimited token
            trimmed.split_whitespace().next().unwrap_or(trimmed)
        });
    unquoted.to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{AppSource, Application};

    fn make_app(name: &str, exec: &str, source: AppSource) -> Application {
        Application {
            name: name.to_string(),
            exec_command: exec.to_string(),
            source,
            location: String::new(),
            icon: None,
            categories: Vec::new(),
            description: None,
        }
    }

    fn make_app_with_desc(
        name: &str,
        exec: &str,
        source: AppSource,
        desc: Option<&str>,
    ) -> Application {
        Application {
            name: name.to_string(),
            exec_command: exec.to_string(),
            source,
            location: String::new(),
            icon: None,
            categories: Vec::new(),
            description: desc.map(|s| s.to_string()),
        }
    }

    #[test]
    fn test_deduplicate_prefers_desktop_over_standalone() {
        let apps = vec![
            make_app("Firefox", "/usr/bin/firefox", AppSource::Standalone),
            make_app("Firefox", "/usr/bin/firefox", AppSource::Desktop),
        ];
        let deduped = DiscoveryEngine::deduplicate(apps);
        assert_eq!(deduped.len(), 1);
        assert_eq!(deduped[0].source, AppSource::Desktop);
    }

    #[test]
    fn test_deduplicate_prefers_more_metadata() {
        let apps = vec![
            make_app("Firefox", "/usr/bin/firefox", AppSource::Desktop),
            make_app_with_desc(
                "Firefox",
                "/usr/bin/firefox",
                AppSource::Desktop,
                Some("Web Browser"),
            ),
        ];
        let deduped = DiscoveryEngine::deduplicate(apps);
        assert_eq!(deduped.len(), 1);
        assert!(deduped[0].description.is_some());
    }

    #[test]
    fn test_deduplicate_different_apps_kept() {
        let apps = vec![
            make_app("Firefox", "/usr/bin/firefox", AppSource::Desktop),
            make_app("GIMP", "/usr/bin/gimp", AppSource::Desktop),
        ];
        let deduped = DiscoveryEngine::deduplicate(apps);
        assert_eq!(deduped.len(), 2);
    }

    #[test]
    fn test_find_by_name_exact() {
        let apps = vec![
            make_app("Firefox", "/usr/bin/firefox", AppSource::Desktop),
            make_app("GIMP", "/usr/bin/gimp", AppSource::Desktop),
        ];
        let engine = DiscoveryEngine::new();
        let found = engine.find_by_name("firefox", &apps);
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Firefox");
    }

    #[test]
    fn test_find_by_name_fuzzy() {
        let apps = vec![
            make_app(
                "Firefox Web Browser",
                "/usr/bin/firefox",
                AppSource::Desktop,
            ),
            make_app("GIMP", "/usr/bin/gimp", AppSource::Desktop),
        ];
        let engine = DiscoveryEngine::new();
        let found = engine.find_by_name("firefox", &apps);
        assert!(found.is_some());
        assert!(found.unwrap().name.contains("Firefox"));
    }

    #[test]
    fn test_search() {
        let apps = vec![
            make_app_with_desc(
                "Firefox",
                "/usr/bin/firefox",
                AppSource::Desktop,
                Some("Web Browser"),
            ),
            make_app("GIMP", "/usr/bin/gimp", AppSource::Desktop),
            make_app("Thunderbird", "/usr/bin/thunderbird", AppSource::Desktop),
        ];
        let engine = DiscoveryEngine::new();
        let results = engine.search("fire", &apps);
        assert!(!results.is_empty());
        assert_eq!(results[0].name, "Firefox");
    }

    #[test]
    fn test_normalize_exec() {
        assert_eq!(normalize_exec("/usr/bin/firefox"), "/usr/bin/firefox");
        assert_eq!(
            normalize_exec("\"/path/with spaces/app\" --arg"),
            "/path/with spaces/app"
        );
        assert_eq!(normalize_exec("  /usr/bin/app  "), "/usr/bin/app");
    }
}
