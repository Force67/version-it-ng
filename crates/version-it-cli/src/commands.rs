use version_it_core::{VersionInfo, Config};
use super::output::{output_success, output_error};
use super::git_ops::{git_commit_changes, git_create_tag};

#[derive(Debug)]
pub struct BumpOptions {
    pub version: Option<String>,
    pub bump: String,
    pub scheme: Option<String>,
    pub channel: Option<String>,
    pub create_tag: bool,
    pub commit: bool,
    pub dry_run: bool,
}

#[derive(Debug)]
pub struct AutoBumpOptions {
    pub create_tag: bool,
    pub commit: bool,
    pub dry_run: bool,
}

#[derive(Debug)]
pub struct CommandContext {
    pub config: Option<Config>,
    pub structured_output: bool,
}

pub fn get_version_info_with_scheme(version: Option<String>, config: &Option<Config>, scheme_override: Option<String>, channel_override: Option<String>) -> Result<VersionInfo, String> {
    let version_str = version.or_else(|| config.as_ref().and_then(|c| c.get_current_version().ok()));

    if version_str.is_none() {
        return Err("No version provided and no config found".to_string());
    }

    let version_str = version_str.unwrap();

    let scheme = scheme_override.or_else(|| config.as_ref().map(|c| c.versioning_scheme.clone())).unwrap_or("semantic".to_string());
    let channel = channel_override.or_else(|| config.as_ref().and_then(|c| c.channel.clone()));
    VersionInfo::new(&version_str, &scheme, channel).map_err(|e| format!("Error parsing version: {}", e))
}

pub fn apply_bump(v: &mut VersionInfo, bump: &str) -> Result<(), String> {
    match bump {
        "major" => {
            v.bump_major();
            Ok(())
        }
        "minor" => {
            v.bump_minor();
            Ok(())
        }
        "patch" => {
            v.bump_patch();
            Ok(())
        }
        _ => Err(format!("Invalid bump type: {}. Use major, minor, or patch.", bump)),
    }
}

pub fn handle_bump_command(options: BumpOptions, context: &CommandContext) {
    let mut v = match get_version_info_with_scheme(options.version, &context.config, options.scheme, options.channel) {
        Ok(v) => v,
        Err(e) => output_error(context.structured_output, &e),
    };
    let old_version = v.to_string();
    if let Err(e) = apply_bump(&mut v, &options.bump) {
        output_error(context.structured_output, &e);
    }

    let new_version = v.to_string();
    if context.structured_output {
        let data = serde_json::json!({
            "success": true,
            "version": new_version,
            "previous_version": old_version,
            "bump_type": options.bump
        });
        output_success(context.structured_output, data);
    } else {
        println!("{}", new_version);
    }

    if options.dry_run {
        println!("DRY RUN: Would perform the following operations:");
        if let Some(ref cfg) = &context.config {
            if let Some(ref file) = cfg.current_version_file {
                println!("  - Write version '{}' to file '{}'", new_version, file);
            }
            if let Some(ref headers) = cfg.version_headers {
                for header in headers {
                    println!("  - Generate header file '{}'", header.path);
                }
            }
            if let Some(ref package_files) = cfg.package_files {
                for package_file in package_files {
                    println!("  - Update version in '{}' ({})", package_file.path, package_file.manager);
                }
            }
        }
        if options.commit {
            println!("  - Commit changes with message 'Bump version to {}'", new_version);
        }
        if options.create_tag {
            println!("  - Create git tag '{}'", new_version);
        }
    } else {
        if let Some(ref cfg) = &context.config {
            if let Some(ref file) = cfg.current_version_file {
                if let Err(e) = std::fs::write(file, &new_version) {
                    output_error(context.structured_output, &format!("Error writing version to file: {}", e));
                }
            }
            if let Err(e) = cfg.generate_headers(&new_version, v.channel.as_deref()) {
                output_error(context.structured_output, &format!("Error generating headers: {}", e));
            }
            if let Err(e) = cfg.update_package_files(&new_version) {
                output_error(context.structured_output, &format!("Error updating package files: {}", e));
            }
        }

        // Git operations
        if options.commit {
            if let Err(e) = git_commit_changes(&new_version) {
                output_error(context.structured_output, &format!("Error committing changes: {}", e));
            }
        }

        if options.create_tag {
            if let Err(e) = git_create_tag(&new_version) {
                output_error(context.structured_output, &format!("Error creating tag: {}", e));
            }
        }
    }
}

pub fn handle_next_command(options: BumpOptions, context: &CommandContext) {
    let mut v = match get_version_info_with_scheme(options.version, &context.config, options.scheme, options.channel) {
        Ok(v) => v,
        Err(e) => output_error(context.structured_output, &e),
    };
    if let Err(e) = apply_bump(&mut v, &options.bump) {
        output_error(context.structured_output, &e);
    }

    let next_version = v.to_string();
    if context.structured_output {
        let data = serde_json::json!({
            "success": true,
            "version": next_version
        });
        output_success(context.structured_output, data);
    } else {
        println!("{}", next_version);
    }
}

pub fn handle_auto_bump_command(options: AutoBumpOptions, context: &CommandContext) {
    if let Some(ref cfg) = &context.config {
        match cfg.analyze_commits_for_bump() {
            Ok(Some(bump_type)) => {
                // Get current version from file or latest tag or config
                let current_version = cfg.get_current_version().unwrap_or_else(|_| {
                    cfg.get_latest_version_tag().unwrap_or(Some(cfg.first_version.clone())).unwrap_or(cfg.first_version.clone())
                });
                let v_result = VersionInfo::new(&current_version, &cfg.versioning_scheme, cfg.channel.clone());

                        if let Err(e) = &v_result {
                            output_error(context.structured_output, &format!("Error parsing version: {}", e));
                        }

                let mut v = v_result.unwrap();

                        match bump_type.as_str() {
                            "major" => v.bump_major(),
                            "minor" => v.bump_minor(),
                            "patch" => v.bump_patch(),
                            _ => {
                                output_error(context.structured_output, &format!("Unknown bump type: {}", bump_type));
                            }
                        }

                        let new_version = v.to_string();
                        if context.structured_output {
                            let data = serde_json::json!({
                                "success": true,
                                "version": new_version,
                                "bump_type": bump_type
                            });
                            output_success(context.structured_output, data);
                        } else {
                            println!("{}", new_version);
                        }

                        if options.dry_run {
                            println!("DRY RUN: Would perform the following operations:");
                            if let Some(ref file) = cfg.current_version_file {
                                println!("  - Write version '{}' to file '{}'", new_version, file);
                            }
                            if let Some(ref headers) = cfg.version_headers {
                                for header in headers {
                                    println!("  - Generate header file '{}'", header.path);
                                }
                            }
                            if let Some(ref package_files) = cfg.package_files {
                                for package_file in package_files {
                                    println!("  - Update version in '{}' ({})", package_file.path, package_file.manager);
                                }
                            }
                            if options.commit {
                                println!("  - Commit changes with message 'Bump version to {}'", new_version);
                            }
                            if options.create_tag {
                                println!("  - Create git tag '{}'", new_version);
                            }
                        } else {
                            if let Some(ref file) = cfg.current_version_file {
                                if let Err(e) = std::fs::write(file, &new_version) {
                                    output_error(context.structured_output, &format!("Error writing version to file: {}", e));
                                }
                            }
                            if let Err(e) = cfg.generate_headers(&new_version, v.channel.as_deref()) {
                                output_error(context.structured_output, &format!("Error generating headers: {}", e));
                            }
                            if let Err(e) = cfg.update_package_files(&new_version) {
                                output_error(context.structured_output, &format!("Error updating package files: {}", e));
                            }

                            // Git operations
                            if options.commit {
                                if let Err(e) = git_commit_changes(&new_version) {
                                    output_error(context.structured_output, &format!("Error committing changes: {}", e));
                                }
                            }

                            if options.create_tag {
                                if let Err(e) = git_create_tag(&new_version) {
                                    output_error(context.structured_output, &format!("Error creating tag: {}", e));
                                }
                            }
                }
            }
                    Ok(None) => {
                        if context.structured_output {
                            let data = serde_json::json!({
                                "success": true,
                                "message": "No bump needed"
                            });
                            output_success(context.structured_output, data);
                        } else {
                            println!("No bump needed");
                        }
                    }
                    Err(e) => {
                        output_error(context.structured_output, &format!("Error analyzing commits: {}", e));
                    }
                }
            } else {
                output_error(context.structured_output, "No config found for auto-bump");
            }
}