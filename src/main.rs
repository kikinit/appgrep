mod app;
mod cli;
mod engine;
mod error;
mod output;
mod provider;

use anyhow::Result;
use clap::Parser;

use cli::{Cli, Command};
use engine::DiscoveryEngine;
use output::Formatter;

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
    }

    Ok(())
}
