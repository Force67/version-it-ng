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