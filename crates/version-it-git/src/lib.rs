use std::process::Command;

pub trait GitManager {
    fn current_commit_full(&self) -> Result<String, Box<dyn std::error::Error>>;
    fn current_commit_short(&self) -> Result<String, Box<dyn std::error::Error>>;
    fn current_branch(&self) -> Result<String, Box<dyn std::error::Error>>;
    fn current_tags(&self) -> Result<Vec<String>, Box<dyn std::error::Error>>;
    fn get_commits_since(&self, since: &str) -> Result<Vec<String>, Box<dyn std::error::Error>>;
    fn get_latest_version_tag(&self, versioning_scheme: &str) -> Result<Option<String>, Box<dyn std::error::Error>>;
}

pub struct DefaultGitManager;

impl GitManager for DefaultGitManager {
    fn current_commit_full(&self) -> Result<String, Box<dyn std::error::Error>> {
        let output = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .output()?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err("Failed to get git commit".into())
        }
    }

    fn current_commit_short(&self) -> Result<String, Box<dyn std::error::Error>> {
        let output = Command::new("git")
            .args(["rev-parse", "--short", "HEAD"])
            .output()?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err("Failed to get git commit".into())
        }
    }

    fn current_branch(&self) -> Result<String, Box<dyn std::error::Error>> {
        let output = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .output()?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err("Failed to get git branch".into())
        }
    }

    fn current_tags(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let output = Command::new("git")
            .args(["tag", "--points-at", "HEAD"])
            .output()?;
        if output.status.success() {
            let tags = String::from_utf8_lossy(&output.stdout);
            Ok(tags.lines().map(|l| l.to_string()).collect())
        } else {
            Ok(vec![])
        }
    }

    fn get_commits_since(&self, since: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let output = Command::new("git")
            .args(["log", "--oneline", &format!("{}..HEAD", since)])
            .output()?;
        if output.status.success() {
            let commits = String::from_utf8_lossy(&output.stdout);
            Ok(commits.lines().map(|l| l.to_string()).collect())
        } else {
            Ok(vec![]) // no commits
        }
    }

    fn get_latest_version_tag(&self, versioning_scheme: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let output = Command::new("git")
            .args(["tag", "--list", "--sort=-version:refname"])
            .output()?;
        if output.status.success() {
            let tags = String::from_utf8_lossy(&output.stdout);
            for tag in tags.lines() {
                if self.is_version_tag(tag, versioning_scheme) {
                    return Ok(Some(tag.to_string()));
                }
            }
        }
        Ok(None)
    }
}

impl DefaultGitManager {
    fn is_version_tag(&self, tag: &str, versioning_scheme: &str) -> bool {
        match versioning_scheme {
            "semantic" => semver::Version::parse(tag).is_ok(),
            "calver" => tag.contains('.') && tag.chars().all(|c| c.is_ascii_digit() || c == '.'),
            _ => true, // for others, assume any tag
        }
    }
}