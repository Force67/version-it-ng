mod output;
mod commands;
mod git_ops;

use clap::{Parser, Subcommand};
use version_it_core::Config;
use std::path::Path;
use output::output_error;
use commands::{handle_bump_command, handle_next_command, handle_auto_bump_command, BumpOptions, AutoBumpOptions, CommandContext};

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
    }
}