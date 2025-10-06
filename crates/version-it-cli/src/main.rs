use clap::{Parser, Subcommand};
use version_it_core::{VersionInfo, Config};
use std::process;
use std::path::Path;
use std::process::Command;

#[derive(Parser)]
#[command(name = "version-it")]
#[command(about = "A semantic versioning tool for CI pipelines")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    /// Path to config file (default: .version-it)
    #[arg(short, long, default_value = ".version-it")]
    config: String,
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
        /// Create a git tag after bumping
        #[arg(long)]
        create_tag: bool,
        /// Commit version file changes after bumping
        #[arg(long)]
        commit: bool,
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
    },
    /// Automatically bump version based on commits
    AutoBump {
        /// Create a git tag after bumping
        #[arg(long)]
        create_tag: bool,
        /// Commit version file changes after bumping
        #[arg(long)]
        commit: bool,
    },
}

fn get_version_info_with_scheme(version: Option<String>, config: &Option<Config>, scheme_override: Option<String>) -> VersionInfo {
    let version_str = version.or_else(|| config.as_ref().and_then(|c| c.get_current_version().ok())).unwrap_or_else(|| {
        eprintln!("No version provided and no config found");
        process::exit(1);
    });

    let scheme = scheme_override.or_else(|| config.as_ref().map(|c| c.versioning_scheme.clone())).unwrap_or_else(|| "semantic".to_string());
    VersionInfo::new(&version_str, &scheme).unwrap_or_else(|e| {
        eprintln!("Error parsing version: {}", e);
        process::exit(1);
    })
}

fn apply_bump(v: &mut VersionInfo, bump: &str) {
    match bump {
        "major" => v.bump_major(),
        "minor" => v.bump_minor(),
        "patch" => v.bump_patch(),
        _ => {
            eprintln!("Invalid bump type: {}. Use major, minor, or patch.", bump);
            process::exit(1);
        }
    }
}

fn git_commit_changes(version: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Add all changes to git
    let status = Command::new("git")
        .args(&["add", "."])
        .status()?;

    if !status.success() {
        return Err("Failed to add files to git".into());
    }

    // Check if there are any changes to commit
    let status_output = Command::new("git")
        .args(&["status", "--porcelain"])
        .output()?;

    if status_output.stdout.is_empty() {
        // No changes to commit
        return Ok(());
    }

    // Commit the changes
    let commit_message = format!("Bump version to {}", version);
    let status = Command::new("git")
        .args(&["commit", "-m", &commit_message])
        .status()?;

    if !status.success() {
        return Err("Failed to commit changes".into());
    }

    println!("Committed version bump: {}", version);
    Ok(())
}

fn git_create_tag(version: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Create an annotated tag
    let tag_message = format!("Version {}", version);
    let status = Command::new("git")
        .args(&["tag", "-a", version, "-m", &tag_message])
        .status()?;

    if !status.success() {
        return Err("Failed to create git tag".into());
    }

    println!("Created git tag: {}", version);
    Ok(())
}

fn main() {
    let cli = Cli::parse();
    let config = if Path::new(&cli.config).exists() {
        match Config::load_from_file(&cli.config) {
            Ok(c) => Some(c),
            Err(e) => {
                eprintln!("Error loading config: {}", e);
                process::exit(1);
            }
        }
    } else {
        None
    };

    let cli = Cli::parse();

    match cli.command {
        Commands::Bump { version, bump, scheme, create_tag, commit } => {
            let mut v = get_version_info_with_scheme(version, &config, scheme);
            apply_bump(&mut v, &bump);

            let new_version = v.to_string();
            println!("{}", new_version);

            if let Some(ref cfg) = config {
                if let Some(ref file) = cfg.current_version_file {
                    if let Err(e) = std::fs::write(file, &new_version) {
                        eprintln!("Error writing version to file: {}", e);
                        process::exit(1);
                    }
                }
                if let Err(e) = cfg.generate_headers(&new_version) {
                    eprintln!("Error generating headers: {}", e);
                    process::exit(1);
                }
                if let Err(e) = cfg.update_package_files(&new_version) {
                    eprintln!("Error updating package files: {}", e);
                    process::exit(1);
                }
            }

            // Git operations
            if commit {
                if let Err(e) = git_commit_changes(&new_version) {
                    eprintln!("Error committing changes: {}", e);
                    process::exit(1);
                }
            }

            if create_tag {
                if let Err(e) = git_create_tag(&new_version) {
                    eprintln!("Error creating tag: {}", e);
                    process::exit(1);
                }
            }
        }
        Commands::Next { version, bump, scheme } => {
            let mut v = get_version_info_with_scheme(version, &config, scheme);
            apply_bump(&mut v, &bump);

            println!("{}", v.to_string());
        }
        Commands::AutoBump { create_tag, commit } => {
            if let Some(ref cfg) = config {
                match cfg.analyze_commits_for_bump() {
                    Ok(Some(bump_type)) => {
                        // Get current version from file or latest tag or config
                        let current_version = cfg.get_current_version().unwrap_or_else(|_| {
                            cfg.get_latest_version_tag().unwrap_or(Some(cfg.first_version.clone())).unwrap_or(cfg.first_version.clone())
                        });
                        let mut v = VersionInfo::new(&current_version, &cfg.versioning_scheme).unwrap_or_else(|e| {
                            eprintln!("Error parsing version: {}", e);
                            process::exit(1);
                        });

                        match bump_type.as_str() {
                            "major" => v.bump_major(),
                            "minor" => v.bump_minor(),
                            "patch" => v.bump_patch(),
                            _ => {
                                eprintln!("Unknown bump type: {}", bump_type);
                                process::exit(1);
                            }
                        }

                        let new_version = v.to_string();
                        println!("{}", new_version);

                        if let Some(ref file) = cfg.current_version_file {
                            if let Err(e) = std::fs::write(file, &new_version) {
                                eprintln!("Error writing version to file: {}", e);
                                process::exit(1);
                            }
                        }
                        if let Err(e) = cfg.generate_headers(&new_version) {
                            eprintln!("Error generating headers: {}", e);
                            process::exit(1);
                        }
                        if let Err(e) = cfg.update_package_files(&new_version) {
                            eprintln!("Error updating package files: {}", e);
                            process::exit(1);
                        }

                        // Git operations
                        if commit {
                            if let Err(e) = git_commit_changes(&new_version) {
                                eprintln!("Error committing changes: {}", e);
                                process::exit(1);
                            }
                        }

                        if create_tag {
                            if let Err(e) = git_create_tag(&new_version) {
                                eprintln!("Error creating tag: {}", e);
                                process::exit(1);
                            }
                        }
                    }
                    Ok(None) => {
                        println!("No bump needed");
                    }
                    Err(e) => {
                        eprintln!("Error analyzing commits: {}", e);
                        process::exit(1);
                    }
                }
            } else {
                eprintln!("No config found for auto-bump");
                process::exit(1);
            }
        }
    }
}