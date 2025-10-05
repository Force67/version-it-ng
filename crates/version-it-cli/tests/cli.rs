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
  - language: c
    path: "{}"
"#, version_file, header_file);
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
    assert!(header.contains("#define VERSION \"1.1.1\""));

    // Clean up
    fs::remove_file(config_path).unwrap();
    fs::remove_file(version_file).unwrap();
    fs::remove_file(header_file).unwrap();
}