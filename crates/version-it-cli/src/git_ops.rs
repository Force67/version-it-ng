use std::process::Command;

pub fn git_commit_changes(version: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Add all changes to git
    let status = Command::new("git")
        .args(["add", "."])
        .status()?;

    if !status.success() {
        return Err("Failed to add files to git".into());
    }

    // Check if there are any changes to commit
    let status_output = Command::new("git")
        .args(["status", "--porcelain"])
        .output()?;

    if status_output.stdout.is_empty() {
        // No changes to commit
        return Ok(());
    }

    // Commit the changes
    let commit_message = format!("Bump version to {}", version);
    let status = Command::new("git")
        .args(["commit", "-m", &commit_message])
        .status()?;

    if !status.success() {
        return Err("Failed to commit changes".into());
    }

    println!("Committed version bump: {}", version);
    Ok(())
}

pub fn git_create_tag(version: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Create an annotated tag
    let tag_message = format!("Version {}", version);
    let status = Command::new("git")
        .args(["tag", "-a", version, "-m", &tag_message])
        .status()?;

    if !status.success() {
        return Err("Failed to create git tag".into());
    }

    println!("Created git tag: {}", version);
    Ok(())
}