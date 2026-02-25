mod app;
mod cli;
mod engine;
mod error;
mod output;
mod provider;

use std::collections::HashMap;

use anyhow::Result;
use clap::{CommandFactory, Parser};
use clap_complete::generate;

use app::AppSource;
use cli::{Cli, Command};
use engine::DiscoveryEngine;
use output::{Formatter, OutputFormat};

fn print_stats(apps: &[app::Application], format: OutputFormat, w: &mut dyn std::io::Write) -> Result<()> {
    let mut counts: HashMap<AppSource, usize> = HashMap::new();
    for app in apps {
        *counts.entry(app.source.clone()).or_insert(0) += 1;
    }

    if format == OutputFormat::Json {
        // For JSON, we print a separate _stats object
        let stats_obj: HashMap<String, usize> = counts
            .iter()
            .map(|(k, v)| (k.to_string(), *v))
            .collect();
        let json = serde_json::to_string_pretty(&stats_obj)?;
        writeln!(w, "{}", json)?;
    } else {
        let sources = [
            AppSource::Desktop,
            AppSource::Flatpak,
            AppSource::Snap,
            AppSource::Standalone,
            AppSource::Cargo,
            AppSource::Npm,
            AppSource::Dpkg,
            AppSource::Rpm,
            AppSource::Pacman,
            AppSource::Brew,
        ];
        let parts: Vec<String> = sources
            .iter()
            .map(|s| format!("{} {}", counts.get(s).unwrap_or(&0), s))
            .collect();
        let total: usize = counts.values().sum();
        eprintln!("Stats: {} — total {}", parts.join(", "), total);
    }

    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let engine = DiscoveryEngine::new();
    let formatter = Formatter::new(cli.format, cli.no_color);

    match cli.command {
        Command::List => {
            let apps = if cli.source.is_empty() {
                engine.discover_all()
            } else {
                engine.discover_filtered(&cli.source)
            };
            formatter.format_list(&apps, &mut std::io::stdout())?;
            if cli.stats {
                print_stats(&apps, cli.format, &mut std::io::stderr())?;
            }
        }
        Command::Info { name } => {
            let apps = engine.discover_all();
            match engine.find_by_name(&name, &apps) {
                Some(app) => {
                    formatter.format_info(&app, &mut std::io::stdout())?;
                }
                None => {
                    eprintln!("Application '{}' not found", name);
                    std::process::exit(1);
                }
            }
        }
        Command::Search { query } => {
            let apps = engine.discover_all();
            let results = engine.search(&query, &apps);
            formatter.format_list(&results, &mut std::io::stdout())?;
            if cli.stats {
                print_stats(&results, cli.format, &mut std::io::stderr())?;
            }
        }
        Command::Has { name } => {
            let apps = engine.discover_all();
            match engine.find_by_name(&name, &apps) {
                Some(app) => {
                    formatter.format_has(&app, true, &mut std::io::stdout())?;
                    std::process::exit(0);
                }
                None => {
                    formatter.format_has_not_found(&name, &mut std::io::stdout())?;
                    std::process::exit(1);
                }
            }
        }
        Command::Run { name } => {
            let apps = engine.discover_all();
            match engine.find_by_name(&name, &apps) {
                Some(app) => {
                    eprintln!("Running: {}", app.exec_command);
                    let parts: Vec<&str> = app.exec_command.split_whitespace().collect();
                    if parts.is_empty() {
                        eprintln!("Empty exec command");
                        std::process::exit(1);
                    }
                    let mut cmd = std::process::Command::new(parts[0]);
                    if parts.len() > 1 {
                        cmd.args(&parts[1..]);
                    }
                    cmd.stdin(std::process::Stdio::null())
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null());
                    match cmd.spawn() {
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!("Failed to launch '{}': {}", app.name, e);
                            std::process::exit(1);
                        }
                    }
                }
                None => {
                    eprintln!("Application '{}' not found", name);
                    std::process::exit(1);
                }
            }
        }
        Command::Path { name } => {
            let apps = engine.discover_all();
            match engine.find_by_name(&name, &apps) {
                Some(app) => {
                    println!("{}", app.exec_command);
                }
                None => {
                    eprintln!("Application '{}' not found", name);
                    std::process::exit(1);
                }
            }
        }
        Command::Doctor => {
            println!("appgrep doctor\n");
            println!("Providers:");

            let mut total = 0;
            for provider in engine.providers() {
                if provider.is_available() {
                    match provider.discover() {
                        Ok(apps) => {
                            let count = apps.len();
                            total += count;
                            let preview: Vec<&str> = apps
                                .iter()
                                .take(3)
                                .map(|a| a.name.as_str())
                                .collect();
                            let preview_str = if preview.is_empty() {
                                String::new()
                            } else {
                                format!("   {}", preview.join(", "))
                            };
                            println!(
                                "  \u{2713} {:<14} {:>3} apps{}",
                                provider.name(),
                                count,
                                preview_str
                            );
                        }
                        Err(e) => {
                            println!(
                                "  \u{2717} {:<14} error: {}",
                                provider.name(),
                                e
                            );
                        }
                    }
                } else {
                    println!(
                        "  \u{2717} {:<14} unavailable",
                        provider.name()
                    );
                }
            }

            println!("\nTotal: {} apps (before dedup)", total);
        }
        Command::Completions { shell } => {
            let mut cmd = Cli::command();
            generate(shell, &mut cmd, "appgrep", &mut std::io::stdout());
        }
    }

    Ok(())
}
