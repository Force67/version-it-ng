use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc, Datelike};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionBlock {
    pub name: String,
    pub block_type: BlockType,
    pub format: Option<String>,
    pub config: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BlockType {
    Semantic { major: Option<u32>, minor: Option<u32>, patch: Option<u32> },
    Calver { year: Option<u32>, month: Option<u32>, day: Option<u32> },
    Timestamp,
    Commit,
    Counter { name: String },
    Text { value: String },
    Date { format: String },
    Branch,
    BuildNumber,
    Versioned { name: String },
}

impl VersionBlock {
    pub fn new(name: &str, block_type: BlockType) -> Self {
        Self {
            name: name.to_string(),
            block_type,
            format: None,
            config: HashMap::new(),
        }
    }

    pub fn with_format(mut self, format: &str) -> Self {
        self.format = Some(format.to_string());
        self
    }

    pub fn with_config(mut self, key: &str, value: &str) -> Self {
        self.config.insert(key.to_string(), value.to_string());
        self
    }

    pub fn generate_value(&self, context: &VersionContext) -> Result<String, Box<dyn std::error::Error>> {
        match &self.block_type {
            BlockType::Semantic { major, minor, patch } => {
                let base_version = context.current_version.as_deref().unwrap_or("0.0.0");
                let mut version = semver::Version::parse(base_version)?;

                if let Some(m) = major { version.major = *m as u64; }
                if let Some(m) = minor { version.minor = *m as u64; }
                if let Some(p) = patch { version.patch = *p as u64; }

                Ok(version.to_string())
            }
            BlockType::Calver { year, month, day } => {
                let now = Utc::now();
                let y = year.unwrap_or(now.year() as u32);
                let m = month.unwrap_or(now.month() as u32);
                let d = day.unwrap_or(now.day() as u32);

                let formatted = match self.format.as_deref() {
                    Some("YY.MM.DD") => format!("{:02}.{:02}.{:02}", y % 100, m, d),
                    Some("YYYY.MM.DD") => format!("{:04}.{:02}.{:02}", y, m, d),
                    Some("YYMMDD") => format!("{:02}{:02}{:02}", y % 100, m, d),
                    Some("YYYYMMDD") => format!("{:04}{:02}{:02}", y, m, d),
                    _ => format!("{:02}.{:02}.{:02}", y % 100, m, d),
                };
                Ok(formatted)
            }
            BlockType::Timestamp => {
                let timestamp = match self.format.as_deref() {
                    Some("unix") => Utc::now().timestamp().to_string(),
                    Some("unix_ms") => Utc::now().timestamp_millis().to_string(),
                    Some("YYYYMMDDHHMMSS") => Utc::now().format("%Y%m%d%H%M%S").to_string(),
                    Some("iso") => Utc::now().to_rfc3339(),
                    _ => Utc::now().format("%Y%m%d%H%M%S").to_string(),
                };
                Ok(timestamp)
            }
            BlockType::Commit => {
                let commit = if let Some(ref c) = context.current_commit {
                    c.clone()
                } else {
                    let output = Command::new("git").args(["rev-parse", "--short", "HEAD"]).output()?;
                    if output.status.success() {
                        String::from_utf8_lossy(&output.stdout).trim().to_string()
                    } else {
                        "unknown".to_string()
                    }
                };
                Ok(commit)
            }
            BlockType::Counter { name } => {
                let counter = context.counters.get(name).copied().unwrap_or(0);
                Ok(counter.to_string())
            }
            BlockType::Text { value } => Ok(value.clone()),
            BlockType::Date { format } => {
                let formatted = Utc::now().format(format).to_string();
                Ok(formatted)
            }
            BlockType::Branch => {
                let branch = if let Some(ref b) = context.current_branch {
                    b.clone()
                } else {
                    let output = Command::new("git").args(["rev-parse", "--abbrev-ref", "HEAD"]).output()?;
                    if output.status.success() {
                        String::from_utf8_lossy(&output.stdout).trim().to_string()
                    } else {
                        "unknown".to_string()
                    }
                };
                Ok(branch)
            }
            BlockType::BuildNumber => {
                let build = context.build_number.unwrap_or(1);
                Ok(build.to_string())
            }
            BlockType::Versioned { name } => {
                // This allows referencing a previously defined version block
                if let Some(value) = context.block_values.get(name) {
                    Ok(value.clone())
                } else {
                    Err(format!("Referenced version block '{}' not found", name).into())
                }
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct VersionContext {
    pub current_version: Option<String>,
    pub current_commit: Option<String>,
    pub current_branch: Option<String>,
    pub build_number: Option<u32>,
    pub counters: HashMap<String, u32>,
    pub block_values: HashMap<String, String>,
}

impl VersionContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_version(mut self, version: &str) -> Self {
        self.current_version = Some(version.to_string());
        self
    }

    pub fn with_commit(mut self, commit: &str) -> Self {
        self.current_commit = Some(commit.to_string());
        self
    }

    pub fn with_branch(mut self, branch: &str) -> Self {
        self.current_branch = Some(branch.to_string());
        self
    }

    pub fn with_build_number(mut self, build: u32) -> Self {
        self.build_number = Some(build);
        self
    }

    pub fn with_counter(mut self, name: &str, value: u32) -> Self {
        self.counters.insert(name.to_string(), value);
        self
    }

    pub fn set_block_value(&mut self, name: &str, value: String) {
        self.block_values.insert(name.to_string(), value);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionTemplate {
    pub name: String,
    pub blocks: Vec<VersionBlock>,
    pub separator: String,
    pub prefix: Option<String>,
    pub suffix: Option<String>,
}

impl VersionTemplate {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            blocks: Vec::new(),
            separator: ".".to_string(),
            prefix: None,
            suffix: None,
        }
    }

    pub fn with_separator(mut self, separator: &str) -> Self {
        self.separator = separator.to_string();
        self
    }

    pub fn with_prefix(mut self, prefix: &str) -> Self {
        self.prefix = Some(prefix.to_string());
        self
    }

    pub fn with_suffix(mut self, suffix: &str) -> Self {
        self.suffix = Some(suffix.to_string());
        self
    }

    pub fn add_block(mut self, block: VersionBlock) -> Self {
        self.blocks.push(block);
        self
    }

    pub fn generate(&self, context: &mut VersionContext) -> Result<String, Box<dyn std::error::Error>> {
        let mut parts = Vec::new();

        // Clear previous block values
        context.block_values.clear();

        // Generate values for each block
        for block in &self.blocks {
            let value = block.generate_value(context)?;
            context.set_block_value(&block.name, value.clone());
            parts.push(value);
        }

        let joined = parts.join(&self.separator);
        let result = match (&self.prefix, &self.suffix) {
            (Some(prefix), Some(suffix)) => format!("{}{}{}", prefix, joined, suffix),
            (Some(prefix), None) => format!("{}{}", prefix, joined),
            (None, Some(suffix)) => format!("{}{}", joined, suffix),
            (None, None) => joined,
        };

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semantic_block() {
        let block = VersionBlock::new(
            "version",
            BlockType::Semantic { major: Some(1), minor: Some(2), patch: Some(3) }
        );
        let context = VersionContext::new();
        assert_eq!(block.generate_value(&context).unwrap(), "1.2.3");
    }

    #[test]
    fn test_calver_block_different_formats() {
        let block = VersionBlock::new(
            "date",
            BlockType::Calver { year: Some(2025), month: Some(10), day: Some(10) }
        );
        let context = VersionContext::new();

        let formatted_block = block.clone().with_format("YYYY.MM.DD");
        assert_eq!(formatted_block.generate_value(&context).unwrap(), "2025.10.10");

        let short_block = block.with_format("YYMMDD");
        assert_eq!(short_block.generate_value(&context).unwrap(), "251010");
    }

    #[test]
    fn test_timestamp_block() {
        let block = VersionBlock::new("timestamp", BlockType::Timestamp);
        let context = VersionContext::new();
        let result = block.generate_value(&context).unwrap();
        assert!(result.len() == 14); // YYYYMMDDHHMMSS format
    }

    #[test]
    fn test_text_block() {
        let block = VersionBlock::new("label", BlockType::Text { value: "alpha".to_string() });
        let context = VersionContext::new();
        assert_eq!(block.generate_value(&context).unwrap(), "alpha");
    }

    #[test]
    fn test_counter_block() {
        let block = VersionBlock::new("counter", BlockType::Counter { name: "build".to_string() });
        let context = VersionContext::new().with_counter("build", 42);
        assert_eq!(block.generate_value(&context).unwrap(), "42");
    }

    #[test]
    fn test_version_template() {
        let template = VersionTemplate::new("test")
            .with_prefix("v")
            .with_separator(".")
            .add_block(
                VersionBlock::new("major", BlockType::Semantic { major: Some(1), minor: Some(0), patch: Some(0) })
            )
            .add_block(
                VersionBlock::new("date", BlockType::Date { format: "%Y%m%d".to_string() })
            )
            .add_block(
                VersionBlock::new("commit", BlockType::Commit)
            );

        let mut context = VersionContext::new();
        let result = template.generate(&mut context);

        assert!(result.is_ok());
        let version = result.unwrap();
        assert!(version.starts_with("v1.0.0."));
        assert!(version.contains('.'));
    }

    #[test]
    fn test_complex_version_template() {
        let template = VersionTemplate::new("complex")
            .with_separator("-")
            .with_prefix("release-")
            .add_block(
                VersionBlock::new("ver", BlockType::Semantic { major: Some(2), minor: Some(1), patch: Some(0) })
            )
            .add_block(
                VersionBlock::new("branch", BlockType::Branch)
            )
            .add_block(
                VersionBlock::new("build", BlockType::Counter { name: "build_counter".to_string() })
            );

        let mut context = VersionContext::new()
            .with_branch("main")
            .with_counter("build_counter", 5);

        let result = template.generate(&mut context).unwrap();
        assert!(result.starts_with("release-2.1.0-"));
        assert!(result.contains("-main-5"));
    }

    #[test]
    fn test_versioned_block_reference() {
        let template = VersionTemplate::new("ref-test")
            .add_block(
                VersionBlock::new("base", BlockType::Semantic { major: Some(1), minor: Some(2), patch: Some(3) })
            )
            .add_block(
                VersionBlock::new("ref", BlockType::Versioned { name: "base".to_string() })
            );

        let mut context = VersionContext::new();
        let result = template.generate(&mut context).unwrap();
        assert_eq!(result, "1.2.3.1.2.3");
    }

    #[test]
    fn test_complex_version_composition() {
        // Test a complex real-world version composition scenario
        let template = VersionTemplate::new("enterprise-release")
            .with_prefix("v")
            .with_separator("-")
            .add_block(
                VersionBlock::new("major", BlockType::Semantic { major: Some(3), minor: Some(0), patch: Some(0) })
            )
            .add_block(
                VersionBlock::new("date", BlockType::Calver { year: None, month: None, day: None })
                    .with_format("YY.MM")
            )
            .add_block(
                VersionBlock::new("release", BlockType::Counter { name: "release".to_string() })
            )
            .add_block(
                VersionBlock::new("commit", BlockType::Commit)
            );

        let mut context = VersionContext::new()
            .with_counter("release", 12)
            .with_commit("abc123");

        let result = template.generate(&mut context).unwrap();

        // Result should be in format: v3.0.0-25.10-12-abc123 (or similar date)
        assert!(result.starts_with("v3.0.0-"));
        assert!(result.contains("-12-"));
        assert!(result.ends_with("abc123"));
    }

    #[test]
    fn test_template_serialization() {
        // Test that templates can be serialized and deserialized
        let template = VersionTemplate::new("test")
            .with_prefix("v")
            .with_separator(".")
            .with_suffix("-beta")
            .add_block(
                VersionBlock::new("version", BlockType::Semantic { major: Some(1), minor: Some(0), patch: Some(0) })
            );

        // Serialize
        let serialized = serde_yaml::to_string(&template).unwrap();
        assert!(!serialized.is_empty());

        // Deserialize
        let deserialized: VersionTemplate = serde_yaml::from_str(&serialized).unwrap();
        assert_eq!(deserialized.name, "test");
        assert_eq!(deserialized.prefix, Some("v".to_string()));
        assert_eq!(deserialized.separator, ".");
        assert_eq!(deserialized.suffix, Some("-beta".to_string()));
        assert_eq!(deserialized.blocks.len(), 1);
    }

    #[test]
    fn test_error_handling() {
        // Test error handling for invalid configurations
        let template = VersionTemplate::new("error-test")
            .add_block(
                VersionBlock::new("ref", BlockType::Versioned { name: "nonexistent".to_string() })
            );

        let mut context = VersionContext::new();
        let result = template.generate(&mut context);

        assert!(result.is_err());
        let error = result.unwrap_err().to_string();
        assert!(error.contains("nonexistent"));
    }
}