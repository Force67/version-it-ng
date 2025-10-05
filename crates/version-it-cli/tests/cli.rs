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
    use std::env;

    // Create a temp directory for the subproject
    let temp_dir = env::temp_dir().join("version_it_test_sub");
    fs::create_dir_all(&temp_dir).unwrap();

    // Create version file
    let version_file = temp_dir.join("sub").join("version.txt");
    fs::write(&version_file, "1.1.0").unwrap();

    // Create subfolder config
    let sub_config = temp_dir.join("sub").join(".version-it");
    fs::create_dir_all(sub_config.parent().unwrap()).unwrap();
    let version_file_path = version_file.to_str().unwrap();
    let header_path = temp_dir.join("sub").join("include").join("version.h").to_str().unwrap().to_string();
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
"#, version_file_path, header_path);
    fs::write(&sub_config, yaml).unwrap();

    // Create include dir
    fs::create_dir_all(temp_dir.join("sub").join("include")).unwrap();

    // Run command from original dir with full config path
    let config_path = sub_config.to_str().unwrap();
    let version_file_path = version_file.to_str().unwrap();

    let output = Command::new("cargo")
        .args(&["run", "--bin", "version-it", "--", "--config", config_path, "bump", "--bump", "patch"])
        .output()
        .expect("Failed to run command");

    if !output.status.success() {
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    }

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "1.1.1");

    // Check if header was generated
    let header_content = fs::read_to_string(temp_dir.join("sub").join("include").join("version.h")).unwrap();
    assert!(header_content.contains("#define VERSION \"1.1.1\""));

    // Check if version file was updated
    let updated_version = fs::read_to_string(&version_file).unwrap();
    assert_eq!(updated_version.trim(), "1.1.1");

    // Clean up
    fs::remove_dir_all(&temp_dir).unwrap();
}