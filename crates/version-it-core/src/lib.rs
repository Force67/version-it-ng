use semver::{Version, Prerelease, BuildMetadata};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::process::Command;


#[derive(Debug, Clone)]
pub enum VersionType {
    Semantic(Version),
    Calver { year: u32, month: u32, day: u32 },
    Timestamp(String),
    Commit(String),
    Build { major: u32, minor: u32, patch: u32, build: u32 },
    Monotonic(u64),
    Datetime(String),
    Pattern(String),
}

#[derive(Debug, Clone)]
pub struct VersionInfo {
    pub scheme: String,
    pub version: VersionType,
    pub channel: Option<String>,
}

impl VersionInfo {
    /// Creates a new VersionInfo instance based on the version string and versioning scheme.
    ///
    /// # Arguments
    ///
    /// * `version` - The version string to parse.
    /// * `scheme` - The versioning scheme: "semantic", "calver", "timestamp", or "commit".
    /// * `channel` - Optional channel name (stable, beta, nightly, etc.)
    ///
    /// # Returns
    ///
    /// A Result containing the VersionInfo or an error if parsing fails.
    pub fn new(version: &str, scheme: &str, channel: Option<String>) -> Result<Self, Box<dyn std::error::Error>> {
        let version_type = match scheme {
            "calver" => {
                let parts: Vec<&str> = version.split('.').collect();
                if parts.len() < 2 {
                    return Err("Calver version must have at least YY.MM".into());
                }
                let year = parts[0].parse()?;
                let month = parts[1].parse()?;
                let day = parts.get(2).map(|s| s.parse()).unwrap_or(Ok(1))?;
                VersionType::Calver { year, month, day }
            }
            "timestamp" => {
                if version.is_empty() {
                    VersionType::Timestamp(Self::current_timestamp())
                } else {
                    VersionType::Timestamp(version.to_string())
                }
            }
            "commit" => {
                if version.is_empty() {
                    VersionType::Commit(Self::current_commit()?)
                } else {
                    VersionType::Commit(version.to_string())
                }
            }
            "build" => {
                let parts: Vec<&str> = version.split('.').collect();
                if parts.len() != 4 {
                    return Err("Build version must be in format major.minor.patch.build".into());
                }
                let major = parts[0].parse()?;
                let minor = parts[1].parse()?;
                let patch = parts[2].parse()?;
                let build = parts[3].parse()?;
                VersionType::Build { major, minor, patch, build }
            }
            "monotonic" => {
                let num: u64 = version.parse()?;
                VersionType::Monotonic(num)
            }
            "datetime" => {
                if version.is_empty() {
                    VersionType::Datetime(Self::current_datetime())
                } else {
                    VersionType::Datetime(version.to_string())
                }
            }
            "pattern" => {
                VersionType::Pattern(version.to_string())
            }
            _ => VersionType::Semantic(Version::parse(version)?),
        };
        Ok(Self {
            scheme: scheme.to_string(),
            version: version_type,
            channel: channel,
        })
    }

    /// Bumps the major version component.
    pub fn bump_major(&mut self) {
        match &mut self.version {
            VersionType::Calver { year, month, day } => {
                *year += 1;
                *month = 1;
                *day = 1;
            }
            VersionType::Semantic(v) => {
                v.major += 1;
                v.minor = 0;
                v.patch = 0;
                v.pre = Prerelease::EMPTY;
                v.build = BuildMetadata::EMPTY;
            }
            VersionType::Timestamp(s) => *s = Self::current_timestamp(),
            VersionType::Commit(s) => *s = Self::current_commit().unwrap_or_else(|_| "unknown".to_string()),
            VersionType::Build { major, minor, patch, .. } => {
                *major += 1;
                *minor = 0;
                *patch = 0;
            }
            VersionType::Monotonic(n) => *n += 1,
            VersionType::Datetime(s) => *s = Self::current_datetime(),
            VersionType::Pattern(s) => *s = format!("{}-updated", s),
        }
    }

    /// Bumps the minor version component.
    pub fn bump_minor(&mut self) {
        match &mut self.version {
            VersionType::Calver { month, day, .. } => {
                *month += 1;
                *day = 1;
            }
            VersionType::Semantic(v) => {
                v.minor += 1;
                v.patch = 0;
                v.pre = Prerelease::EMPTY;
                v.build = BuildMetadata::EMPTY;
            }
            VersionType::Timestamp(s) => *s = Self::current_timestamp(),
            VersionType::Commit(s) => *s = Self::current_commit().unwrap_or_else(|_| "unknown".to_string()),
            VersionType::Build { minor, patch, .. } => {
                *minor += 1;
                *patch = 0;
            }
            VersionType::Monotonic(n) => *n += 1,
            VersionType::Datetime(s) => *s = Self::current_datetime(),
            VersionType::Pattern(s) => *s = format!("{}-updated", s),
        }
    }

    /// Bumps the patch version component.
    pub fn bump_patch(&mut self) {
        match &mut self.version {
            VersionType::Calver { day, .. } => {
                *day += 1;
            }
            VersionType::Semantic(v) => {
                v.patch += 1;
                v.pre = Prerelease::EMPTY;
                v.build = BuildMetadata::EMPTY;
            }
            VersionType::Timestamp(s) => *s = Self::current_timestamp(),
            VersionType::Commit(s) => *s = Self::current_commit().unwrap_or_else(|_| "unknown".to_string()),
            VersionType::Build { patch, build, .. } => {
                *patch += 1;
                *build = 0; // reset build on patch bump?
            }
            VersionType::Monotonic(n) => *n += 1,
            VersionType::Datetime(s) => *s = Self::current_datetime(),
            VersionType::Pattern(s) => *s = format!("{}-updated", s),
        }
    }

    pub fn set_prerelease(&mut self, pre: &str) {
        if let VersionType::Semantic(v) = &mut self.version {
            v.pre = Prerelease::new(pre).unwrap_or(Prerelease::EMPTY);
        }
    }

    pub fn set_build(&mut self, build: &str) {
        if let VersionType::Semantic(v) = &mut self.version {
            v.build = BuildMetadata::new(build).unwrap_or(BuildMetadata::EMPTY);
        }
    }

    /// Returns the version as a string.
    pub fn to_string(&self) -> String {
        let base_version = match &self.version {
            VersionType::Calver { year, month, day } => format!("{:02}.{:02}.{:02}", year, month, day),
            VersionType::Semantic(v) => v.to_string(),
            VersionType::Timestamp(s) => s.clone(),
            VersionType::Commit(s) => s.clone(),
            VersionType::Build { major, minor, patch, build } => format!("{}.{}.{}.{}", major, minor, patch, build),
            VersionType::Monotonic(n) => n.to_string(),
            VersionType::Datetime(s) => s.clone(),
            VersionType::Pattern(s) => s.clone(),
        };

        if let Some(ref channel) = self.channel {
            match channel.as_str() {
                "stable" => base_version,
                "beta" => {
                    if let VersionType::Semantic(ref v) = self.version {
                        if v.pre.is_empty() {
                            format!("{}-beta.1", base_version)
                        } else {
                            base_version
                        }
                    } else {
                        format!("{}-beta", base_version)
                    }
                }
                "nightly" => {
                    if matches!(self.version, VersionType::Timestamp(_) | VersionType::Commit(_)) {
                        base_version
                    } else {
                        format!("{}-nightly", base_version)
                    }
                }
                _ => format!("{}-{}", base_version, channel),
            }
        } else {
            base_version
        }
    }

    fn current_timestamp() -> String {
        let now: DateTime<Utc> = Utc::now();
        now.format("%Y%m%d%H%M%S").to_string()
    }

    fn current_commit() -> Result<String, Box<dyn std::error::Error>> {
        let output = Command::new("git").args(&["rev-parse", "--short", "HEAD"]).output()?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err("Failed to get git commit".into())
        }
    }

    fn current_datetime() -> String {
        let now: DateTime<Utc> = Utc::now();
        now.format("%Y-%m-%dT%H:%M:%S").to_string()
    }


}

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
    #[serde(rename = "channel")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
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
        let output = Command::new("git").args(&["rev-parse", "--abbrev-ref", "HEAD"]).output()?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err("Failed to get current branch".into())
        }
    }

    pub fn get_latest_version_tag(&self) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let output = Command::new("git").args(&["tag", "--list", "--sort=-version:refname"]).output()?;
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
            "semantic" => Version::parse(tag).is_ok(),
            "calver" => tag.contains('.') && tag.chars().all(|c| c.is_digit(10) || c == '.'),
            _ => true, // for others, assume any tag
        }
    }

    fn get_commits_since(&self, since: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let output = Command::new("git").args(&["log", "--oneline", &format!("{}..HEAD", since)]).output()?;
        if output.status.success() {
            let commits = String::from_utf8_lossy(&output.stdout);
            Ok(commits.lines().map(|l| l.to_string()).collect())
        } else {
            Ok(vec![]) // no commits
        }
    }

    fn determine_bump_from_commit(&self, commit: &str) -> Option<String> {
        // Simple check: look for labels in commit message
        for map in &self.change_type_map {
            if commit.contains(&map.label) {
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

    /// Generates version header files based on the configuration.
    ///
    /// # Arguments
    ///
    /// * `version` - The version string to include in the headers.
    /// * `channel` - Optional channel name to include in the headers.
    ///
    /// # Returns
    ///
    /// A Result indicating success or failure.
    pub fn generate_headers(&self, version: &str, channel: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(headers) = &self.version_headers {
            let handlebars = handlebars::Handlebars::new();
            for header in headers {
                let template = if let Some(ref template_path) = header.template_path {
                    std::fs::read_to_string(template_path)?
                } else if let Some(ref template) = header.template {
                    template.clone()
                } else {
                    return Err("Either template or template-path must be specified for version header".into());
                };
                let data = serde_json::json!({
                    "version": version,
                    "scheme": self.versioning_scheme,
                    "channel": channel.unwrap_or("")
                });
                let content = handlebars.render_template(&template, &data)?;
                std::fs::write(&header.path, content)?;
            }
        }
        Ok(())
    }

    /// Updates package files with the new version.
    ///
    /// # Arguments
    ///
    /// * `version` - The version string to set in package files.
    ///
    /// # Returns
    ///
    /// A Result indicating success or failure.
    pub fn update_package_files(&self, version: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(package_files) = &self.package_files {
            for package_file in package_files {
                self.update_single_package_file(package_file, version)?;
            }
        }
        Ok(())
    }

    fn update_single_package_file(&self, package_file: &PackageFile, version: &str) -> Result<(), Box<dyn std::error::Error>> {
        if !std::path::Path::new(&package_file.path).exists() {
            // Skip files that don't exist
            return Ok(());
        }
        let content = std::fs::read_to_string(&package_file.path)?;
        let updated_content = match package_file.manager.as_str() {
            "npm" | "yarn" | "pnpm" => self.update_json_file(&content, version, package_file.field.as_deref().unwrap_or("version"))?,
            "cargo" => self.update_toml_file(&content, version, package_file.field.as_deref().unwrap_or("version"))?,
            "python" => self.update_python_file(&content, version, package_file.field.as_deref().unwrap_or("__version__"))?,
            "maven" => self.update_xml_file(&content, version, package_file.field.as_deref().unwrap_or("version"))?,
            _ => return Err(format!("Unsupported package manager: {}", package_file.manager).into()),
        };
        std::fs::write(&package_file.path, updated_content)?;
        Ok(())
    }

    fn update_json_file(&self, content: &str, version: &str, field: &str) -> Result<String, Box<dyn std::error::Error>> {
        let mut json: serde_json::Value = serde_json::from_str(content)?;
        if let Some(obj) = json.as_object_mut() {
            obj.insert(field.to_string(), serde_json::Value::String(version.to_string()));
        }
        Ok(serde_json::to_string_pretty(&json)?)
    }

    fn update_toml_file(&self, content: &str, version: &str, field: &str) -> Result<String, Box<dyn std::error::Error>> {
        let mut toml_value: toml::Value = toml::from_str(content)?;
        if let Some(table) = toml_value.as_table_mut() {
            table.insert(field.to_string(), toml::Value::String(version.to_string()));
        }
        Ok(toml::to_string(&toml_value)?)
    }

    fn update_python_file(&self, content: &str, version: &str, field: &str) -> Result<String, Box<dyn std::error::Error>> {
        let lines: Vec<&str> = content.lines().collect();
        let mut updated_lines = Vec::new();
        let assignment_pattern = format!("{} = ", field);

        for line in lines {
            if line.trim().starts_with(&assignment_pattern) {
                // Simple string assignment replacement
                if let Some(quote_start) = line.find('"').or_else(|| line.find('\'')) {
                    if let Some(quote_end) = line[quote_start + 1..].find(line.chars().nth(quote_start).unwrap()).map(|i| i + quote_start + 1) {
                        let before = &line[..quote_start + 1];
                        let after = &line[quote_end..];
                        updated_lines.push(format!("{}{}{}", before, version, after));
                        continue;
                    }
                }
            }
            updated_lines.push(line.to_string());
        }

        Ok(updated_lines.join("\n"))
    }

    fn update_xml_file(&self, content: &str, version: &str, field: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Simple XML version update - this is a basic implementation
        // For more complex XML structures, a proper XML parser would be better
        let version_tag = format!("<{}>{}</{}>", field, version, field);
        let pattern = format!("<{}>[^<]*</{}>", regex::escape(field), regex::escape(field));

        let re = regex::Regex::new(&pattern)?;
        Ok(re.replace_all(content, version_tag).to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bump_major() {
        let mut v = VersionInfo::new("1.2.3", "semantic", None).unwrap();
        v.bump_major();
        assert_eq!(v.to_string(), "2.0.0");
    }

    #[test]
    fn test_bump_minor() {
        let mut v = VersionInfo::new("1.2.3", "semantic", None).unwrap();
        v.bump_minor();
        assert_eq!(v.to_string(), "1.3.0");
    }

    #[test]
    fn test_bump_patch() {
        let mut v = VersionInfo::new("1.2.3", "semantic", None).unwrap();
        v.bump_patch();
        assert_eq!(v.to_string(), "1.2.4");
    }

    #[test]
    fn test_calver_bump_minor() {
        let mut v = VersionInfo::new("25.10.01", "calver", None).unwrap();
        v.bump_minor();
        assert_eq!(v.to_string(), "25.11.01");
    }

    #[test]
    fn test_calver_bump_major() {
        let mut v = VersionInfo::new("25.10.01", "calver", None).unwrap();
        v.bump_major();
        assert_eq!(v.to_string(), "26.01.01");
    }

    #[test]
    fn test_timestamp_new() {
        let v = VersionInfo::new("", "timestamp", None).unwrap();
        assert!(v.to_string().len() == 14); // YYYYMMDDHHMMSS
    }

    #[test]
    fn test_commit_new() {
        // This will fail if no git, but assume it's there
        let v = VersionInfo::new("", "commit", None);
        if let Ok(v) = v {
            assert!(v.to_string().len() > 0);
        }
    }

    #[test]
    fn test_versioninfo_new_semantic() {
        let v = VersionInfo::new("1.2.3", "semantic", None).unwrap();
        assert_eq!(v.scheme, "semantic");
        assert_eq!(v.to_string(), "1.2.3");
    }

    #[test]
    fn test_versioninfo_new_calver() {
        let v = VersionInfo::new("25.10.01", "calver", None).unwrap();
        assert_eq!(v.scheme, "calver");
        assert_eq!(v.to_string(), "25.10.01");
    }

    #[test]
    fn test_versioninfo_new_timestamp() {
        let v = VersionInfo::new("20231005120000", "timestamp", None).unwrap();
        assert_eq!(v.scheme, "timestamp");
        assert_eq!(v.to_string(), "20231005120000");
    }

    #[test]
    fn test_versioninfo_new_commit() {
        let v = VersionInfo::new("abc123", "commit", None).unwrap();
        assert_eq!(v.scheme, "commit");
        assert_eq!(v.to_string(), "abc123");
    }

    #[test]
    fn test_versioninfo_new_invalid_semantic() {
        let result = VersionInfo::new("invalid", "semantic", None);
        assert!(result.is_err());
    }

    #[test]
    fn test_versioninfo_new_invalid_calver() {
        let result = VersionInfo::new("25", "calver", None);
        assert!(result.is_err());
    }

    #[test]
    fn test_set_prerelease() {
        let mut v = VersionInfo::new("1.2.3", "semantic", None).unwrap();
        v.set_prerelease("alpha.1");
        assert_eq!(v.to_string(), "1.2.3-alpha.1");
    }

    #[test]
    fn test_set_build() {
        let mut v = VersionInfo::new("1.2.3", "semantic", None).unwrap();
        v.set_build("build.1");
        assert_eq!(v.to_string(), "1.2.3+build.1");
    }

    #[test]
    fn test_set_prerelease_and_build() {
        let mut v = VersionInfo::new("1.2.3", "semantic", None).unwrap();
        v.set_prerelease("beta");
        v.set_build("sha.123");
        assert_eq!(v.to_string(), "1.2.3-beta+sha.123");
    }

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
            channel: None,
        };
        let version = config.get_current_version().unwrap();
        assert_eq!(version, "2.1.0");
        fs::remove_file("test_version.txt").unwrap();
    }

    #[test]
    fn test_get_current_version_fallback() {
        let config = Config {
            run_on_branches: vec![],
            versioning_scheme: "semantic".to_string(),
            first_version: "1.0.0".to_string(),
            current_version_file: None,
            changelog_exporters: None,
            calver_enable_branch: false,
            changelog_sections: vec![],
            change_substitutions: vec![],
            change_type_map: vec![],
            version_headers: None,
            package_files: None,
            channel: None,
        };
        let version = config.get_current_version().unwrap();
        assert_eq!(version, "1.0.0");
    }

    #[test]
    fn test_config_load_invalid_file() {
        let result = Config::load_from_file("nonexistent.yml");
        assert!(result.is_err());
    }

    #[test]
    fn test_config_load_invalid_yaml() {
        use std::fs;
        fs::write("invalid.yml", "invalid: yaml: [").unwrap();
        let result = Config::load_from_file("invalid.yml");
        assert!(result.is_err());
        fs::remove_file("invalid.yml").unwrap();
    }

    #[test]
    fn test_generate_headers() {
        use std::fs;
        let config = Config {
            run_on_branches: vec!["main".to_string()],
            versioning_scheme: "semantic".to_string(),
            first_version: "1.0.0".to_string(),
            current_version_file: None,
            changelog_exporters: None,
            calver_enable_branch: false,
            changelog_sections: vec![],
            change_substitutions: vec![],
            change_type_map: vec![],
            version_headers: Some(vec![
                VersionHeader {
                    path: "test_version.h".to_string(),
                    template: Some("#define VERSION \"{{version}}\"\n".to_string()),
                    template_path: None,
                },
                VersionHeader {
                    path: "test_version.py".to_string(),
                    template: Some("VERSION = \"{{version}}\"".to_string()),
                    template_path: None,
                },
            ]),
            package_files: None,
            channel: None,
        };
        config.generate_headers("2.0.0", None).unwrap();
        let c_content = fs::read_to_string("test_version.h").unwrap();
        assert_eq!(c_content, "#define VERSION \"2.0.0\"\n");
        let py_content = fs::read_to_string("test_version.py").unwrap();
        assert_eq!(py_content, "VERSION = \"2.0.0\"");
        fs::remove_file("test_version.h").unwrap();
        fs::remove_file("test_version.py").unwrap();
    }

    #[test]
    fn test_generate_headers_no_headers() {
        let config = Config {
            run_on_branches: vec![],
            versioning_scheme: "semantic".to_string(),
            first_version: "1.0.0".to_string(),
            current_version_file: None,
            changelog_exporters: None,
            calver_enable_branch: false,
            changelog_sections: vec![],
            change_substitutions: vec![],
            change_type_map: vec![],
            version_headers: None,
            package_files: None,
            channel: None,
        };
        let result = config.generate_headers("2.0.0", None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_analyze_commits_for_bump_no_git() {
        let config = Config {
            run_on_branches: vec![],
            versioning_scheme: "semantic".to_string(),
            first_version: "1.0.0".to_string(),
            current_version_file: None,
            changelog_exporters: None,
            calver_enable_branch: false,
            changelog_sections: vec![],
            change_substitutions: vec![],
            change_type_map: vec![],
            version_headers: None,
            package_files: None,
            channel: None,
        };
        // Since no git repo, get_current_branch will fail, so should return Err or None
        let result = config.analyze_commits_for_bump();
        // Depending on implementation, it might be Err
        assert!(result.is_err() || result.unwrap().is_none());
    }

    #[test]
    fn test_versioninfo_new_build() {
        let v = VersionInfo::new("1.2.3.4", "build", None).unwrap();
        assert_eq!(v.scheme, "build");
        assert_eq!(v.to_string(), "1.2.3.4");
    }

    #[test]
    fn test_versioninfo_new_monotonic() {
        let v = VersionInfo::new("42", "monotonic", None).unwrap();
        assert_eq!(v.scheme, "monotonic");
        assert_eq!(v.to_string(), "42");
    }

    #[test]
    fn test_channel_stable() {
        let v = VersionInfo::new("1.2.3", "semantic", Some("stable".to_string())).unwrap();
        assert_eq!(v.to_string(), "1.2.3");
    }

    #[test]
    fn test_channel_beta() {
        let v = VersionInfo::new("1.2.3", "semantic", Some("beta".to_string())).unwrap();
        assert_eq!(v.to_string(), "1.2.3-beta.1");
    }

    #[test]
    fn test_channel_nightly() {
        let v = VersionInfo::new("20241006", "timestamp", Some("nightly".to_string())).unwrap();
        assert_eq!(v.to_string(), "20241006");
    }

    #[test]
    fn test_channel_custom() {
        let v = VersionInfo::new("1.2.3", "semantic", Some("rc".to_string())).unwrap();
        assert_eq!(v.to_string(), "1.2.3-rc");
    }

    #[test]
    fn test_versioninfo_new_datetime() {
        let v = VersionInfo::new("2024-10-06T14:30:00", "datetime", None).unwrap();
        assert_eq!(v.scheme, "datetime");
        assert_eq!(v.to_string(), "2024-10-06T14:30:00");
    }

    #[test]
    fn test_versioninfo_new_pattern() {
        let v = VersionInfo::new("v1.0.0-snapshot", "pattern", None).unwrap();
        assert_eq!(v.scheme, "pattern");
        assert_eq!(v.to_string(), "v1.0.0-snapshot");
    }

    #[test]
    fn test_build_bump_major() {
        let mut v = VersionInfo::new("1.2.3.4", "build", None).unwrap();
        v.bump_major();
        assert_eq!(v.to_string(), "2.0.0.4");
    }

    #[test]
    fn test_build_bump_minor() {
        let mut v = VersionInfo::new("1.2.3.4", "build", None).unwrap();
        v.bump_minor();
        assert_eq!(v.to_string(), "1.3.0.4");
    }

    #[test]
    fn test_build_bump_patch() {
        let mut v = VersionInfo::new("1.2.3.4", "build", None).unwrap();
        v.bump_patch();
        assert_eq!(v.to_string(), "1.2.4.0");
    }

    #[test]
    fn test_monotonic_bump() {
        let mut v = VersionInfo::new("42", "monotonic", None).unwrap();
        v.bump_major();
        assert_eq!(v.to_string(), "43");
        v.bump_minor();
        assert_eq!(v.to_string(), "44");
        v.bump_patch();
        assert_eq!(v.to_string(), "45");
    }

    #[test]
    fn test_datetime_bump() {
        let mut v = VersionInfo::new("2024-10-06T14:30:00", "datetime", None).unwrap();
        let original = v.to_string();
        v.bump_major();
        assert_ne!(v.to_string(), original); // should update to current time
    }

    #[test]
    fn test_pattern_bump() {
        let mut v = VersionInfo::new("v1.0.0", "pattern", None).unwrap();
        v.bump_major();
        assert_eq!(v.to_string(), "v1.0.0-updated");
    }
}