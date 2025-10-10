use version_it_core::{VersionInfo, Config, VersionComposer, ComposerConfig, VersionContext};
use super::output::{output_success, output_error};
use super::git_ops::{git_commit_changes, git_create_tag};
use std::thread;
use std::path::Path;

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
pub struct CraftOptions {
    pub template: Option<String>,
    pub config_file: Option<String>,
    pub list_templates: bool,
    pub increment_counter: Option<String>,
    pub set_counter: Option<(String, u32)>,
    pub dry_run: bool,
}

#[derive(Debug)]
pub struct MonorepoOptions {
    pub bump: String,
    pub create_tag: bool,
    pub commit: bool,
    pub dry_run: bool,
    pub parallel: bool,
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

pub fn handle_craft_command(options: CraftOptions, context: &CommandContext) {
    // Load version composer configuration
    let config_file = options.config_file.as_deref().unwrap_or("version-templates.yaml");

    let composer_config = match ComposerConfig::from_file(config_file) {
        Ok(config) => config,
        Err(e) => {
            output_error(context.structured_output, &format!("Error loading template config '{}': {}", config_file, e));
            return;
        }
    };

    let mut composer = VersionComposer::from_config(&composer_config);

    // Handle listing templates
    if options.list_templates {
        let templates = composer.list_templates();
        if context.structured_output {
            let data = serde_json::json!({
                "success": true,
                "templates": templates,
                "default_template": composer.default_template
            });
            output_success(context.structured_output, data);
        } else {
            println!("Available templates:");
            for template_name in templates {
                let default_marker = if composer.default_template.as_ref().map_or(false, |d| d == template_name) {
                    " (default)"
                } else {
                    ""
                };
                println!("  {}{}", template_name, default_marker);
            }
        }
        return;
    }

    // Handle counter operations
    if let Some(counter_name) = options.increment_counter {
        if options.dry_run {
            println!("DRY RUN: Would increment counter '{}' from {} to {}",
                     counter_name,
                     composer.counters.get(&counter_name).copied().unwrap_or(0),
                     composer.counters.get(&counter_name).copied().unwrap_or(0) + 1);
        } else {
            let new_value = composer.increment_counter(&counter_name);
            if context.structured_output {
                let data = serde_json::json!({
                    "success": true,
                    "counter": counter_name,
                    "new_value": new_value
                });
                output_success(context.structured_output, data);
            } else {
                println!("Counter '{}' incremented to {}", counter_name, new_value);
            }
        }
        return;
    }

    if let Some((counter_name, value)) = options.set_counter {
        if options.dry_run {
            println!("DRY RUN: Would set counter '{}' to {}", counter_name, value);
        } else {
            composer.set_counter(&counter_name, value);
            if context.structured_output {
                let data = serde_json::json!({
                    "success": true,
                    "counter": counter_name,
                    "value": value
                });
                output_success(context.structured_output, data);
            } else {
                println!("Counter '{}' set to {}", counter_name, value);
            }
        }
        return;
    }

    // Generate version
    match composer.generate_version(options.template.as_deref()) {
        Ok(version) => {
            if context.structured_output {
                let data = serde_json::json!({
                    "success": true,
                    "version": version,
                    "template": options.template.or(composer.default_template),
                    "counters": composer.counters
                });
                output_success(context.structured_output, data);
            } else {
                println!("{}", version);
            }
        }
        Err(e) => {
            output_error(context.structured_output, &format!("Error generating version: {}", e));
        }
    }
}

fn process_subproject(subproject_path: String, subproject_config_path: Option<String>, bump_type: String, dry_run: bool, root_dir: std::path::PathBuf) -> (String, Result<String, String>) {
    // Construct absolute path to subproject
    let subproject_abs_path = root_dir.join(&subproject_path);

    // Load subproject config
    let mut sub_config = if let Some(config_path) = subproject_config_path {
        // If config path is relative, make it absolute from the subproject directory
        let config_abs_path = if Path::new(&config_path).is_relative() {
            subproject_abs_path.join(&config_path)
        } else {
            Path::new(&config_path).to_path_buf()
        };
        match Config::load_from_file(config_abs_path.to_str().unwrap_or(&config_path)) {
            Ok(config) => config,
            Err(e) => {
                return (subproject_path, Err(format!("Config error: {}", e)));
            }
        }
    } else {
        // Try default .version-it in subproject directory
        let config_abs_path = subproject_abs_path.join(".version-it");
        match Config::load_from_file(config_abs_path.to_str().unwrap()) {
            Ok(config) => config,
            Err(e) => {
                return (subproject_path, Err(format!("Config error: {}", e)));
            }
        }
    };

    // Override the base_path to point to the subproject directory
    sub_config.base_path = subproject_abs_path.clone();

    // Get current version
    let current_version_info = match get_version_info_with_scheme(None, &Some(sub_config.clone()), None, None) {
        Ok(version_info) => version_info,
        Err(e) => {
            return (subproject_path, Err(format!("Version error: {}", e)));
        }
    };

    let current_version = current_version_info.to_string();

    // Calculate next version
    let mut next_version_info = current_version_info.clone();
    if let Err(e) = apply_bump(&mut next_version_info, &bump_type) {
        return (subproject_path, Err(format!("Bump error: {}", e)));
    }

    let next_version = next_version_info.to_string();

    if !dry_run {
        // Apply the bump by updating package files and generating headers
        if let Err(e) = sub_config.update_package_files(&next_version) {
            return (subproject_path, Err(format!("Package update error: {}", e)));
        }

        if let Err(e) = sub_config.generate_headers(&next_version, None) {
            return (subproject_path, Err(format!("Header generation error: {}", e)));
        }

        // Update the version file
        if let Some(ref version_file) = sub_config.current_version_file {
            let version_file_path = subproject_abs_path.join(version_file);
            if let Err(e) = std::fs::write(&version_file_path, &next_version) {
                return (subproject_path, Err(format!("Version file write error: {}", e)));
            }
        }

        // Print the new version (mimicking handle_bump_command output)
        println!("{}", next_version);
    }

    (subproject_path, Ok(next_version))
}

pub fn handle_monorepo_command(options: MonorepoOptions, context: &CommandContext) {
    if options.dry_run {
        println!("🔍 DRY RUN - No changes will be made");
    }

    // Get the root config
    let root_config = match &context.config {
        Some(config) => config,
        None => {
            output_error(context.structured_output, "No root .version-it config found");
            return;
        }
    };

    // Check if subprojects are defined
    let subprojects = match &root_config.subprojects {
        Some(projects) => projects,
        None => {
            output_error(context.structured_output, "No subprojects defined in root .version-it config. Add a 'subprojects' section.");
            return;
        }
    };

    if subprojects.is_empty() {
        output_error(context.structured_output, "No subprojects defined in root .version-it config");
        return;
    }

    println!("🚀 Processing {} subprojects with bump: {} ({})",
             subprojects.len(),
             options.bump,
             if options.parallel { "parallel" } else { "sequential" });

    let mut results = Vec::new();

    if options.parallel {
        // Get the root directory before spawning threads
        let root_dir = match std::env::current_dir() {
            Ok(dir) => dir,
            Err(e) => {
                output_error(context.structured_output, &format!("Failed to get current directory: {}", e));
                return;
            }
        };

        // Parallel processing using threads
        let mut handles = Vec::new();

        for subproject in subprojects {
            let subproject_path = subproject.path.clone();
            let subproject_config_path = subproject.config.clone();
            let bump_type = options.bump.clone();
            let dry_run = options.dry_run;
            let root_dir_clone = root_dir.clone();

            let handle = std::thread::spawn(move || {
                process_subproject(subproject_path, subproject_config_path, bump_type, dry_run, root_dir_clone)
            });

            handles.push(handle);
        }

        // Collect results from all threads
        for handle in handles {
            match handle.join() {
                Ok(result) => results.push(result),
                Err(e) => {
                    println!("  ❌ Thread panicked: {:?}", e);
                    results.push(("unknown".to_string(), Err("Thread panic".to_string())));
                }
            }
        }
    } else {
        // Sequential processing (original logic)
        for subproject in subprojects {
            println!("\n📦 Processing: {}", subproject.path);

            // Change to subproject directory
            let original_dir = std::env::current_dir().unwrap();
            if let Err(e) = std::env::set_current_dir(&subproject.path) {
                println!("  ❌ Failed to change to directory {}: {}", subproject.path, e);
                results.push((subproject.path.clone(), Err(format!("Directory error: {}", e))));
                continue;
            }

            // Load subproject config
            let sub_config = if let Some(config_path) = &subproject.config {
                match Config::load_from_file(config_path) {
                    Ok(config) => Some(config),
                    Err(e) => {
                        println!("  ❌ Failed to load config {}: {}", config_path, e);
                        results.push((subproject.path.clone(), Err(format!("Config error: {}", e))));
                        let _ = std::env::set_current_dir(original_dir);
                        continue;
                    }
                }
            } else {
                // Try default .version-it in subproject directory
                match Config::load_from_file(".version-it") {
                    Ok(config) => Some(config),
                    Err(e) => {
                        println!("  ❌ Failed to load config .version-it: {}", e);
                        results.push((subproject.path.clone(), Err(format!("Config error: {}", e))));
                        let _ = std::env::set_current_dir(original_dir);
                        continue;
                    }
                }
            };

            // Get current version
            let current_version_info = match get_version_info_with_scheme(None, &sub_config, None, None) {
                Ok(version_info) => version_info,
                Err(e) => {
                    println!("  ❌ Failed to get current version: {}", e);
                    results.push((subproject.path.clone(), Err(format!("Version error: {}", e))));
                    let _ = std::env::set_current_dir(original_dir);
                    continue;
                }
            };

            let current_version = current_version_info.to_string();
            println!("  📋 Current version: {}", current_version);

            // Calculate next version
            let mut next_version_info = current_version_info.clone();
            if let Err(e) = apply_bump(&mut next_version_info, &options.bump) {
                println!("  ❌ Failed to bump version: {}", e);
                results.push((subproject.path.clone(), Err(format!("Bump error: {}", e))));
                let _ = std::env::set_current_dir(original_dir);
                continue;
            }

            let next_version = next_version_info.to_string();
            println!("  🎯 Next version: {}", next_version);

            if !options.dry_run {
                // Apply the bump
                let bump_options = BumpOptions {
                    version: Some(current_version.clone()),
                    bump: options.bump.clone(),
                    scheme: None,
                    channel: None,
                    create_tag: false, // We'll handle tagging at the end if requested
                    commit: false,     // We'll handle committing at the end if requested
                    dry_run: false,
                };

                // We need to create a new context for the subproject
                let sub_context = CommandContext {
                    config: sub_config,
                    structured_output: false, // Disable structured output for subprojects
                };

                handle_bump_command(bump_options, &sub_context);
                println!("  ✅ Bumped to: {}", next_version);
            } else {
                println!("  🔍 Would bump to: {}", next_version);
            }

            results.push((subproject.path.clone(), Ok(next_version.clone())));

            // Return to original directory
            let _ = std::env::set_current_dir(original_dir);
        }
    }

    // Summary
    println!("\n📊 Monorepo bump summary:");
    let mut success_count = 0;
    let mut error_count = 0;

    for (path, result) in &results {
        match result {
            Ok(version) => {
                println!("  ✅ {}: {}", path, version);
                success_count += 1;
            }
            Err(error) => {
                println!("  ❌ {}: {}", path, error);
                error_count += 1;
            }
        }
    }

    println!("\n📈 Results: {} successful, {} failed", success_count, error_count);

    if !options.dry_run && (options.commit || options.create_tag) {
        println!("\n🔄 Handling git operations...");

        if options.commit {
            println!("  📝 Committing all changes...");
            if let Ok(root_version) = root_config.get_current_version() {
                if let Err(e) = git_commit_changes(&root_version) {
                    output_error(context.structured_output, &format!("Failed to commit changes: {}", e));
                    return;
                }
            }
        }

        if options.create_tag {
            // Use the root version for the tag
            if let Ok(root_version) = root_config.get_current_version() {
                println!("  🏷️  Creating tag: v{}", root_version);
                if let Err(e) = git_create_tag(&format!("v{}", root_version)) {
                    output_error(context.structured_output, &format!("Failed to create tag: {}", e));
                    return;
                }
            }
        }
    }

    if context.structured_output {
        let data = serde_json::json!({
            "success": error_count == 0,
            "total_projects": results.len(),
            "successful": success_count,
            "failed": error_count,
            "results": results.into_iter().map(|(path, result)| {
                match result {
                    Ok(version) => serde_json::json!({"path": path, "version": version, "success": true}),
                    Err(error) => serde_json::json!({"path": path, "error": error, "success": false})
                }
            }).collect::<Vec<_>>()
        });
        output_success(context.structured_output, data);
    }
}