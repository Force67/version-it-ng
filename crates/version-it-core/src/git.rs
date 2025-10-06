use std::process::Command;
use serde_json;

impl super::Config {
    fn current_commit_full() -> Result<String, Box<dyn std::error::Error>> {
        let output = Command::new("git").args(["rev-parse", "HEAD"]).output()?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err("Failed to get git commit".into())
        }
    }

    fn current_branch() -> Result<String, Box<dyn std::error::Error>> {
        let output = Command::new("git").args(["rev-parse", "--abbrev-ref", "HEAD"]).output()?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err("Failed to get git branch".into())
        }
    }

    fn latest_tag() -> Result<String, Box<dyn std::error::Error>> {
        let output = Command::new("git").args(["describe", "--tags", "--abbrev=0"]).output()?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Ok("".to_string()) // No tags found
        }
    }

    fn commit_author() -> Result<String, Box<dyn std::error::Error>> {
        let output = Command::new("git").args(["log", "-1", "--pretty=format:%an"]).output()?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err("Failed to get commit author".into())
        }
    }

    fn commit_email() -> Result<String, Box<dyn std::error::Error>> {
        let output = Command::new("git").args(["log", "-1", "--pretty=format:%ae"]).output()?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err("Failed to get commit email".into())
        }
    }

    fn commit_date() -> Result<String, Box<dyn std::error::Error>> {
        let output = Command::new("git").args(["log", "-1", "--pretty=format:%ci"]).output()?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err("Failed to get commit date".into())
        }
    }

    fn recent_commits(limit: usize) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
        let output = Command::new("git")
            .args(["log", &format!("-{}", limit), "--oneline", "--pretty=format:%H|%h|%s|%an|%ae|%ci"])
            .output()?;

        if !output.status.success() {
            return Ok(vec![]);
        }

        let commits = String::from_utf8_lossy(&output.stdout);
        let mut result = Vec::new();

        for line in commits.lines() {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 6 {
                result.push(serde_json::json!({
                    "hash_full": parts[0],
                    "hash_short": parts[1],
                    "subject": parts[2],
                    "author": parts[3],
                    "email": parts[4],
                    "date": parts[5]
                }));
            }
        }

        Ok(result)
    }

    fn commit_count() -> Result<u64, Box<dyn std::error::Error>> {
        let output = Command::new("git")
            .args(["rev-list", "--count", "HEAD"])
            .output()?;

        if output.status.success() {
            let count = String::from_utf8_lossy(&output.stdout).trim().parse().unwrap_or(0);
            Ok(count)
        } else {
            Ok(0)
        }
    }

    fn first_commit_date() -> Result<String, Box<dyn std::error::Error>> {
        let output = Command::new("git")
            .args(["log", "--reverse", "--pretty=format:%ci", "-1"])
            .output()?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Ok("unknown".to_string())
        }
    }

    pub fn gather_git_info() -> serde_json::Value {
        let commit_hash = super::VersionInfo::current_commit().unwrap_or_else(|_| "unknown".to_string());
        let commit_hash_full = Self::current_commit_full().unwrap_or_else(|_| "unknown".to_string());
        let branch = Self::current_branch().unwrap_or_else(|_| "unknown".to_string());
        let tag = Self::latest_tag().unwrap_or_else(|_| "".to_string());
        let author = Self::commit_author().unwrap_or_else(|_| "unknown".to_string());
        let email = Self::commit_email().unwrap_or_else(|_| "unknown".to_string());
        let date = Self::commit_date().unwrap_or_else(|_| "unknown".to_string());
        let commit_count = Self::commit_count().unwrap_or(0);
        let first_commit_date = Self::first_commit_date().unwrap_or_else(|_| "unknown".to_string());
        let recent_commits = Self::recent_commits(10).unwrap_or_else(|_| vec![]);

        serde_json::json!({
            "commit_hash": commit_hash,
            "commit_hash_full": commit_hash_full,
            "branch": branch,
            "tag": tag,
            "author": author,
            "email": email,
            "date": date,
            "commit_count": commit_count,
            "first_commit_date": first_commit_date,
            "recent_commits": recent_commits
        })
    }
}