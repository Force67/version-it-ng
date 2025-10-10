use std::process::Command;

#[test]
fn test_cli_bump_patch() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "version-it", "--", "bump", "--version", "1.0.0", "--bump", "patch"])
        .output()
        .expect("Failed to run command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "1.0.1");
}

#[test]
fn test_cli_next_minor() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "version-it", "--", "next", "--version", "1.0.0", "--bump", "minor"])
        .output()
        .expect("Failed to run command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "1.1.0");
}

#[test]
fn test_cli_bump_with_scheme() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "version-it", "--", "bump", "--version", "1.2.3.4", "--scheme", "build", "--bump", "patch"])
        .output()
        .expect("Failed to run command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "1.2.4.0");
}

#[test]
fn test_cli_structured_input() {
    let config_json = r#"{
        "run-on-branches": ["main"],
        "versioning-scheme": "semantic",
        "first-version": "1.0.0",
        "calver-enable-branch": false,
        "changelog-sections": [],
        "change-substitutions": [],
        "change-type-map": [],
        "commit-based-bumping": false,
        "enable-expensive-metrics": false
    }"#;

    let output = Command::new("cargo")
        .args(&[
            "run", "--bin", "version-it", "--",
            "--structured-input", config_json,
            "bump", "--version", "1.0.0", "--bump", "patch"
        ])
        .output()
        .expect("Failed to run command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "1.0.1");
}

#[test]
fn test_cli_structured_input_with_channel() {
    let config_json = r#"{
        "run-on-branches": ["main", "develop"],
        "versioning-scheme": "semantic",
        "first-version": "1.0.0",
        "calver-enable-branch": false,
        "changelog-sections": [
            {
                "title": "Features",
                "labels": ["feat"]
            }
        ],
        "change-substitutions": [],
        "change-type-map": [
            {
                "label": "feat",
                "action": "minor"
            }
        ],
        "commit-based-bumping": false,
        "enable-expensive-metrics": false,
        "channel": "beta"
    }"#;

    let output = Command::new("cargo")
        .args(&[
            "run", "--bin", "version-it", "--",
            "--structured-input", config_json,
            "bump", "--version", "1.0.0", "--bump", "minor"
        ])
        .output()
        .expect("Failed to run command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "1.1.0-beta");
}

#[test]
fn test_cli_structured_input_with_subprojects() {
    use std::fs;

    // Create temporary subproject directories and files
    fs::create_dir_all("test_sub1").unwrap();
    fs::create_dir_all("test_sub2").unwrap();

    // Create version files for subprojects
    fs::write("test_sub1/version.txt", "1.0.0").unwrap();
    fs::write("test_sub2/version.txt", "2.0.0").unwrap();

    // Create subproject configs
    let sub1_config = r#"
run-on-branches: ["main"]
versioning-scheme: semantic
first-version: "1.0.0"
current-version-file: "version.txt"
calver-enable-branch: false
changelog-sections: []
change-substitutions: []
change-type-map: []
commit-based-bumping: false
enable-expensive-metrics: false
"#;
    fs::write("test_sub1/.version-it", sub1_config).unwrap();

    let sub2_config = r#"
run-on-branches: ["main"]
versioning-scheme: semantic
first-version: "2.0.0"
current-version-file: "version.txt"
calver-enable-branch: false
changelog-sections: []
change-substitutions: []
change-type-map: []
commit-based-bumping: false
enable-expensive-metrics: false
"#;
    fs::write("test_sub2/.version-it", sub2_config).unwrap();

    // JSON config with subprojects
    let config_json = r#"{
        "run-on-branches": ["main"],
        "versioning-scheme": "semantic",
        "first-version": "1.0.0",
        "calver-enable-branch": false,
        "changelog-sections": [],
        "change-substitutions": [],
        "change-type-map": [],
        "commit-based-bumping": false,
        "enable-expensive-metrics": false,
        "subprojects": [
            {"path": "test_sub1"},
            {"path": "test_sub2", "config": "test_sub2/.version-it"}
        ]
    }"#;

    // Test monorepo command with structured input
    let output = Command::new("cargo")
        .args(&[
            "run", "--bin", "version-it", "--",
            "--structured-input", config_json,
            "monorepo", "--bump", "minor", "--dry-run"
        ])
        .output()
        .expect("Failed to run command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("DRY RUN"));
    assert!(stdout.contains("test_sub1"));
    assert!(stdout.contains("test_sub2"));

    // Clean up
    fs::remove_dir_all("test_sub1").unwrap();
    fs::remove_dir_all("test_sub2").unwrap();
}

#[test]
fn test_subfolder_config() {
    use std::fs;

    // Create test files in current directory
    let config_path = "test_sub_config.yml";
    let version_file = "test_version.txt";
    let header_file = "test_version.h";

    // Clean up any existing
    fs::remove_file(config_path).ok();
    fs::remove_file(version_file).ok();
    fs::remove_file(header_file).ok();

    // Write version file
    fs::write(version_file, "1.1.0").unwrap();

    // Write config
    let template = format!("#define VERSION {{{{version}}}}");
    let yaml = format!(r#"
run-on-branches: ["main"]
versioning-scheme: semantic
first-version: "1.0.0"
current-version-file: "{}"
calver-enable-branch: false
changelog-sections: []
change-substitutions: []
change-type-map: []
version-headers:
  - path: "{}"
    template: "{}"
commit-based-bumping: false
enable-expensive-metrics: false
"#, version_file, header_file, template);
    fs::write(config_path, yaml).unwrap();

    // Run command
    let output = Command::new("cargo")
        .args(&["run", "--bin", "version-it", "--", "--config", config_path, "bump", "--bump", "patch"])
        .output()
        .expect("Failed to run command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "1.1.1");

    // Check version file updated
    let updated = fs::read_to_string(version_file).unwrap();
    assert_eq!(updated.trim(), "1.1.1");

    // Check header generated
    let header = fs::read_to_string(header_file).unwrap();
    assert!(header.contains("#define VERSION 1.1.1"));

    // Clean up
    fs::remove_file(config_path).unwrap();
    fs::remove_file(version_file).unwrap();
    fs::remove_file(header_file).unwrap();
}