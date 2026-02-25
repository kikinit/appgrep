use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum AppSource {
    Desktop,
    Flatpak,
    Snap,
    Standalone,
}

impl AppSource {
    /// Priority for deduplication: lower number = higher priority.
    pub fn priority(&self) -> u8 {
        match self {
            AppSource::Desktop => 0,
            AppSource::Flatpak => 1,
            AppSource::Snap => 2,
            AppSource::Standalone => 3,
        }
    }
}

impl fmt::Display for AppSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppSource::Desktop => write!(f, "desktop"),
            AppSource::Flatpak => write!(f, "flatpak"),
            AppSource::Snap => write!(f, "snap"),
            AppSource::Standalone => write!(f, "standalone"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Application {
    pub name: String,
    pub exec_command: String,
    pub source: AppSource,
    pub location: String,
    pub icon: Option<String>,
    pub categories: Vec<String>,
    pub description: Option<String>,
}

impl Application {
    /// Count how many optional metadata fields are populated.
    pub fn metadata_richness(&self) -> usize {
        let mut count = 0;
        if self.icon.is_some() {
            count += 1;
        }
        if !self.categories.is_empty() {
            count += 1;
        }
        if self.description.is_some() {
            count += 1;
        }
        count
    }
}

impl Ord for Application {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name
            .to_lowercase()
            .cmp(&other.name.to_lowercase())
            .then_with(|| self.source.priority().cmp(&other.source.priority()))
    }
}

impl PartialOrd for Application {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
