#![allow(clippy::expect_used)]

use cargo_lint_extra::config::Config;
use cargo_lint_extra::diagnostic::RuleLevel;
use cargo_lint_extra::engine::Engine;
use clap::{Parser, ValueEnum};
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(name = "cargo", bin_name = "cargo")]
enum Cli {
    /// Extra configurable linting rules for Rust projects
    LintExtra(Args),
}

#[derive(Parser)]
struct Args {
    /// Root directory to lint (defaults to current directory)
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Output format
    #[arg(long, default_value = "human")]
    format: OutputFormat,

    /// Path to config file
    #[arg(long)]
    config: Option<PathBuf>,

    /// Enable specific rules (comma-separated)
    #[arg(long, value_delimiter = ',')]
    enable: Vec<String>,

    /// Disable specific rules (comma-separated)
    #[arg(long, value_delimiter = ',')]
    disable: Vec<String>,
}

#[derive(Clone, ValueEnum)]
enum OutputFormat {
    Human,
    Json,
}

#[allow(clippy::option_if_let_else)]
fn main() {
    let Cli::LintExtra(args) = Cli::parse();

    let root = match args.path.canonicalize() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error: invalid path '{}': {e}", args.path.display());
            process::exit(2);
        }
    };

    let mut config = if let Some(config_path) = &args.config {
        match std::fs::read_to_string(config_path) {
            Ok(content) => match toml::from_str(&content) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("error: failed to parse config: {e}");
                    process::exit(2);
                }
            },
            Err(e) => {
                eprintln!(
                    "error: failed to read config '{}': {e}",
                    config_path.display()
                );
                process::exit(2);
            }
        }
    } else {
        match Config::load(&root) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("error: {e}");
                process::exit(2);
            }
        }
    };

    apply_overrides(&mut config, &args.enable, &args.disable);

    let engine = Engine::new(&config);
    let diagnostics = engine.run(&root);

    if diagnostics.is_empty() {
        process::exit(0);
    }

    match args.format {
        OutputFormat::Human => {
            for diag in &diagnostics {
                println!("{}", diag.format_human());
            }
        }
        OutputFormat::Json => {
            let json =
                serde_json::to_string_pretty(&diagnostics).expect("serialization should not fail");
            println!("{json}");
        }
    }

    process::exit(1);
}

fn apply_overrides(config: &mut Config, enable: &[String], disable: &[String]) {
    for rule in enable {
        set_rule_level(config, rule, RuleLevel::Warn);
    }
    for rule in disable {
        set_rule_level(config, rule, RuleLevel::Allow);
    }
}

fn set_rule_level(config: &mut Config, rule: &str, level: RuleLevel) {
    match rule {
        "line-length" => config.rules.line_length.level = level,
        "file-length" => config.rules.file_length.level = level,
        "todo-comments" => config.rules.todo_comments.level = level,
        "file-header" => config.rules.file_header.level = level,
        "allow-audit" => config.rules.allow_audit.level = level,
        _ => eprintln!("warning: unknown rule '{rule}'"),
    }
}
