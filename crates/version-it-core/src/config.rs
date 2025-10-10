use serde::{Deserialize, Serialize};
use std::process::Command;
use regex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangelogExporters {
    #[serde(rename = "template-path")]
    pub template_path: String,
    #[serde(rename = "output-path")]
    pub output_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangelogSection {
    pub title: String,
    pub labels: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeSubstitution {
    pub token: String,
    pub substitution: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChangeAction {
    Null,
    Minor,
    Patch,
    Major,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeTypeMap {
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
    pub action: ChangeAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionHeader {
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
    #[serde(rename = "template-path", skip_serializing_if = "Option::is_none")]
    pub template_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageFile {
    pub path: String,
    pub manager: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subproject {
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(rename = "run-on-branches")]
    pub run_on_branches: Vec<String>,
    #[serde(rename = "versioning-scheme")]
    pub versioning_scheme: String,
    #[serde(rename = "first-version")]
    pub first_version: String,
    #[serde(rename = "current-version-file")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_version_file: Option<String>,
    #[serde(rename = "changelog-exporters")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub changelog_exporters: Option<ChangelogExporters>,
    #[serde(rename = "calver-enable-branch")]
    pub calver_enable_branch: bool,
    #[serde(rename = "changelog-sections")]
    pub changelog_sections: Vec<ChangelogSection>,
    #[serde(rename = "change-substitutions")]
    pub change_substitutions: Vec<ChangeSubstitution>,
    #[serde(rename = "change-type-map")]
    pub change_type_map: Vec<ChangeTypeMap>,
    #[serde(rename = "version-headers")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version_headers: Option<Vec<VersionHeader>>,
    #[serde(rename = "package-files")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_files: Option<Vec<PackageFile>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subprojects: Option<Vec<Subproject>>,
    #[serde(rename = "channel")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
    #[serde(rename = "commit-based-bumping")]
    pub commit_based_bumping: bool,
    #[serde(rename = "enable-expensive-metrics")]
    pub enable_expensive_metrics: bool,
    #[serde(rename = "structured-output", default)]
    pub structured_output: bool,
}

impl Config {
    /// Loads configuration from a YAML file.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the YAML configuration file.
    ///
    /// # Returns
    ///
    /// A Result containing the Config or an error if loading/parsing fails.
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = std::fs::read_to_string(path)?;
        let config: Config = serde_yaml::from_str(&contents)?;
        Ok(config)
    }

    pub fn get_current_version(&self) -> Result<String, Box<dyn std::error::Error>> {
        if let Some(ref file) = self.current_version_file {
            let version = std::fs::read_to_string(file)?;
            Ok(version.trim().to_string())
        } else {
            Ok(self.first_version.clone())
        }
    }

    /// Analyzes recent commits to determine if a version bump is needed.
    ///
    /// # Returns
    ///
    /// A Result containing Some(bump_type) if a bump is needed, None otherwise, or an error.
    pub fn analyze_commits_for_bump(&self) -> Result<Option<String>, Box<dyn std::error::Error>> {
        // Check if commit-based bumping is enabled
        if !self.commit_based_bumping {
            return Ok(None);
        }

        // Check if current branch is allowed
        let current_branch = self.get_current_branch()?;
        if !self.run_on_branches.contains(&current_branch) {
            return Ok(None);
        }

        // Find latest version tag
        let latest_tag = self.get_latest_version_tag()?;
        let since = latest_tag.as_deref().unwrap_or("HEAD~1");

        // Get commits since last tag
        let commits = self.get_commits_since(since)?;

        // Analyze commits for bump type
        let mut bump_type: Option<String> = None;
        for commit in commits {
            if let Some(bt) = self.determine_bump_from_commit(&commit) {
                bump_type = self.higher_bump(bump_type.as_deref(), Some(&bt));
            }
        }

        Ok(bump_type)
    }

    fn get_current_branch(&self) -> Result<String, Box<dyn std::error::Error>> {
        let output = Command::new("git").args(["rev-parse", "--abbrev-ref", "HEAD"]).output()?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err("Failed to get current branch".into())
        }
    }

    pub fn get_latest_version_tag(&self) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let output = Command::new("git").args(["tag", "--list", "--sort=-version:refname"]).output()?;
        if output.status.success() {
            let tags = String::from_utf8_lossy(&output.stdout);
            for tag in tags.lines() {
                if self.is_version_tag(tag) {
                    return Ok(Some(tag.to_string()));
                }
            }
        }
        Ok(None)
    }

    fn is_version_tag(&self, tag: &str) -> bool {
        match self.versioning_scheme.as_str() {
            "semantic" => semver::Version::parse(tag).is_ok(),
            "calver" => tag.contains('.') && tag.chars().all(|c| c.is_ascii_digit() || c == '.'),
            _ => true, // for others, assume any tag
        }
    }

    fn get_commits_since(&self, since: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let output = Command::new("git").args(["log", "--oneline", &format!("{}..HEAD", since)]).output()?;
        if output.status.success() {
            let commits = String::from_utf8_lossy(&output.stdout);
            Ok(commits.lines().map(|l| l.to_string()).collect())
        } else {
            Ok(vec![]) // no commits
        }
    }

    fn determine_bump_from_commit(&self, commit: &str) -> Option<String> {
        // Check for labels/patterns in commit message
        for map in &self.change_type_map {
            let matches = if let Some(ref pattern) = map.pattern {
                // Use regex matching
                if let Ok(re) = regex::Regex::new(pattern) {
                    re.is_match(commit)
                } else {
                    // If regex is invalid, fall back to simple contains
                    commit.contains(&map.label)
                }
            } else {
                // Use simple string contains for backward compatibility
                commit.contains(&map.label)
            };

            if matches {
                match map.action {
                    ChangeAction::Minor => return Some("minor".to_string()),
                    ChangeAction::Patch => return Some("patch".to_string()),
                    ChangeAction::Major => return Some("major".to_string()),
                    ChangeAction::Null => {},
                }
            }
        }
        None
    }

    fn higher_bump(&self, a: Option<&str>, b: Option<&str>) -> Option<String> {
        match (a, b) {
            (None, None) => None,
            (Some(x), None) => Some(x.to_string()),
            (None, Some(y)) => Some(y.to_string()),
            (Some(x), Some(y)) => {
                let order = ["patch", "minor", "major"];
                let x_idx = order.iter().position(|&s| s == x);
                let y_idx = order.iter().position(|&s| s == y);
                match (x_idx, y_idx) {
                    (Some(xi), Some(yi)) => if yi > xi { Some(y.to_string()) } else { Some(x.to_string()) },
                    _ => Some(x.to_string()),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_load_from_file() {
        use std::fs;
        let yaml = r#"
run-on-branches: ["main"]
versioning-scheme: semantic
first-version: "1.0.0"
current-version-file: version.txt
calver-enable-branch: false
changelog-sections:
  - title: Features
    labels: ["feat"]
change-substitutions: []
change-type-map:
  - label: "feat"
    action: minor
version-headers:
  - path: include/version.h
    template: |
      #define VERSION "{{version}}"
commit-based-bumping: false
enable-expensive-metrics: false
"#;
        fs::write("test_config.yml", yaml).unwrap();
        let config = Config::load_from_file("test_config.yml").unwrap();
        assert_eq!(config.versioning_scheme, "semantic");
        assert_eq!(config.first_version, "1.0.0");
        fs::remove_file("test_config.yml").unwrap();
    }

    #[test]
    fn test_get_current_version_from_file() {
        use std::fs;
        fs::write("test_version.txt", "2.1.0\n").unwrap();
        let config = Config {
            run_on_branches: vec![],
            versioning_scheme: "semantic".to_string(),
            first_version: "1.0.0".to_string(),
            current_version_file: Some("test_version.txt".to_string()),
            changelog_exporters: None,
            calver_enable_branch: false,
            changelog_sections: vec![],
            change_substitutions: vec![],
            change_type_map: vec![],
            version_headers: None,
            package_files: None,
            subprojects: None,
            channel: None,
            commit_based_bumping: false,
            enable_expensive_metrics: false,
            structured_output: false,
        };
        let version = config.get_current_version().unwrap();
        assert_eq!(version, "2.1.0");
        fs::remove_file("test_version.txt").unwrap();
    }

    #[test]
    fn test_determine_bump_from_commit_with_regex() {
        let config = Config {
            run_on_branches: vec![],
            versioning_scheme: "semantic".to_string(),
            first_version: "1.0.0".to_string(),
            current_version_file: None,
            changelog_exporters: None,
            calver_enable_branch: false,
            changelog_sections: vec![],
            change_substitutions: vec![],
            change_type_map: vec![
                ChangeTypeMap {
                    label: "feat".to_string(),
                    pattern: Some(r"feat.*".to_string()),
                    action: ChangeAction::Minor,
                },
                ChangeTypeMap {
                    label: "fix".to_string(),
                    pattern: Some(r"fix.*bug".to_string()),
                    action: ChangeAction::Patch,
                },
            ],
            version_headers: None,
            package_files: None,
            subprojects: None,
            channel: None,
            commit_based_bumping: true,
            enable_expensive_metrics: false,
            structured_output: false,
        };

        assert_eq!(config.determine_bump_from_commit("feat: add new feature"), Some("minor".to_string()));
        assert_eq!(config.determine_bump_from_commit("fix: critical bug fix"), Some("patch".to_string()));
        assert_eq!(config.determine_bump_from_commit("fix: typo fix"), None);
    }
}