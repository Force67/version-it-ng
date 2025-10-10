mod output;
mod commands;
mod git_ops;

use clap::{Parser, Subcommand};
use version_it_config::Config;
use std::path::Path;
use output::output_error;
use commands::{handle_bump_command, handle_next_command, handle_auto_bump_command, handle_craft_command, handle_monorepo_command, BumpOptions, AutoBumpOptions, CraftOptions, MonorepoOptions, CommandContext};

#[derive(Parser)]
#[command(name = "version-it")]
#[command(about = "A semantic versioning tool for CI pipelines")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    /// Path to config file (default: .version-it)
    #[arg(short, long, default_value = ".version-it")]
    config: String,
    /// Output responses in structured JSON format
    #[arg(long)]
    structured_output: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Bump the version
    Bump {
        /// Current version (optional, uses config first-version if not provided)
        #[arg(short, long)]
        version: Option<String>,
        /// Bump type: major, minor, patch
        #[arg(short, long)]
        bump: String,
        /// Versioning scheme (optional, uses config or defaults to semantic)
        #[arg(short, long)]
        scheme: Option<String>,
        /// Release channel (stable, beta, nightly, or custom)
        #[arg(long)]
        channel: Option<String>,
        /// Create a git tag after bumping
        #[arg(long)]
        create_tag: bool,
        /// Commit version file changes after bumping
        #[arg(long)]
        commit: bool,
        /// Show what would happen without making changes
        #[arg(long)]
        dry_run: bool,
    },
    /// Get the next version without bumping
    Next {
        /// Current version (optional, uses config first-version if not provided)
        #[arg(short, long)]
        version: Option<String>,
        /// Bump type: major, minor, patch
        #[arg(short, long)]
        bump: String,
        /// Versioning scheme (optional, uses config or defaults to semantic)
        #[arg(short, long)]
        scheme: Option<String>,
        /// Release channel (stable, beta, nightly, or custom)
        #[arg(long)]
        channel: Option<String>,
    },
    /// Automatically bump version based on commits
    AutoBump {
        /// Create a git tag after bumping
        #[arg(long)]
        create_tag: bool,
        /// Commit version file changes after bumping
        #[arg(long)]
        commit: bool,
        /// Show what would happen without making changes
        #[arg(long)]
        dry_run: bool,
    },
    /// Craft custom versions using configurable templates
    Craft {
        /// Template name to use (optional, uses default template if not provided)
        #[arg(short, long)]
        template: Option<String>,
        /// Path to template configuration file (default: version-templates.yaml)
        #[arg(long)]
        config_file: Option<String>,
        /// List all available templates
        #[arg(long)]
        list_templates: bool,
        /// Increment a counter by name
        #[arg(long)]
        increment_counter: Option<String>,
        /// Set a counter to a specific value (format: counter_name:value)
        #[arg(long, value_parser = parse_counter_set)]
        set_counter: Option<(String, u32)>,
        /// Show what would happen without making changes
        #[arg(long)]
        dry_run: bool,
    },
    /// Process multiple subprojects in a monorepo
    Monorepo {
        /// Bump type: major, minor, patch
        #[arg(short, long)]
        bump: String,
        /// Create a git tag after bumping
        #[arg(long)]
        create_tag: bool,
        /// Commit version file changes after bumping
        #[arg(long)]
        commit: bool,
        /// Show what would happen without making changes
        #[arg(long)]
        dry_run: bool,
        /// Process subprojects in parallel
        #[arg(long)]
        parallel: bool,
    },
}

fn parse_counter_set(s: &str) -> Result<(String, u32), String> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 2 {
        return Err("Expected format: counter_name:value".to_string());
    }
    let value = parts[1].parse().map_err(|_| "Invalid number".to_string())?;
    Ok((parts[0].to_string(), value))
}

fn main() {
    let cli = Cli::parse();
    let config = if Path::new(&cli.config).exists() {
        let c = Config::load_from_file(&cli.config);
        if c.is_err() {
            output_error(cli.structured_output, &format!("Error loading config: {}", c.err().unwrap()));
        }
        Some(c.unwrap())
    } else {
        None
    };

    let structured_output = cli.structured_output || config.as_ref().map(|c| c.structured_output).unwrap_or(false);

    let context = CommandContext {
        config,
        structured_output,
    };

    match cli.command {
        Commands::Bump { version, bump, scheme, channel, create_tag, commit, dry_run } => {
            let options = BumpOptions {
                version,
                bump,
                scheme,
                channel,
                create_tag,
                commit,
                dry_run,
            };
            handle_bump_command(options, &context);
        }
        Commands::Next { version, bump, scheme, channel } => {
            let options = BumpOptions {
                version,
                bump,
                scheme,
                channel,
                create_tag: false,
                commit: false,
                dry_run: false,
            };
            handle_next_command(options, &context);
        }
        Commands::AutoBump { create_tag, commit, dry_run } => {
            let options = AutoBumpOptions {
                create_tag,
                commit,
                dry_run,
            };
            handle_auto_bump_command(options, &context);
        }
        Commands::Craft { template, config_file, list_templates, increment_counter, set_counter, dry_run } => {
            let options = CraftOptions {
                template,
                config_file,
                list_templates,
                increment_counter,
                set_counter,
                dry_run,
            };
            handle_craft_command(options, &context);
        }
        Commands::Monorepo { bump, create_tag, commit, dry_run, parallel } => {
            let options = MonorepoOptions {
                bump,
                create_tag,
                commit,
                dry_run,
                parallel,
            };
            handle_monorepo_command(options, &context);
        }
    }
}