use clap::{Parser, Subcommand, ValueEnum};

use crate::app::AppSource;
use crate::output::OutputFormat;

/// Parse a source string into an AppSource.
fn parse_source(s: &str) -> Result<AppSource, String> {
    match s.to_lowercase().as_str() {
        "desktop" => Ok(AppSource::Desktop),
        "flatpak" => Ok(AppSource::Flatpak),
        "snap" => Ok(AppSource::Snap),
        "standalone" => Ok(AppSource::Standalone),
        _ => Err(format!(
            "invalid source '{}': expected desktop, flatpak, snap, or standalone",
            s
        )),
    }
}

#[derive(Parser, Debug)]
#[command(
    name = "appgrep",
    version,
    about = "Unified CLI tool to discover all installed applications on Linux"
)]
pub struct Cli {
    /// Output format
    #[arg(short, long, value_enum, default_value = "table")]
    pub format: OutputFormat,

    /// Filter by source (repeatable)
    #[arg(short, long, value_parser = parse_source)]
    pub source: Vec<AppSource>,

    /// Disable colored output
    #[arg(long)]
    pub no_color: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// List all discovered applications
    List,

    /// Show detailed info about an application
    Info {
        /// Application name
        name: String,
    },

    /// Fuzzy search for applications
    Search {
        /// Search query
        query: String,
    },

    /// Check if an application is installed (exit 0=yes, 1=no)
    Has {
        /// Application name
        name: String,
    },

    /// Launch an application
    Run {
        /// Application name
        name: String,
    },

    /// Print exec command for an application
    Path {
        /// Application name
        name: String,
    },
}

impl ValueEnum for OutputFormat {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            OutputFormat::Table,
            OutputFormat::Json,
            OutputFormat::Tsv,
            OutputFormat::Names,
            OutputFormat::Exec,
        ]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        match self {
            OutputFormat::Table => Some(clap::builder::PossibleValue::new("table")),
            OutputFormat::Json => Some(clap::builder::PossibleValue::new("json")),
            OutputFormat::Tsv => Some(clap::builder::PossibleValue::new("tsv")),
            OutputFormat::Names => Some(clap::builder::PossibleValue::new("names")),
            OutputFormat::Exec => Some(clap::builder::PossibleValue::new("exec")),
        }
    }
}
