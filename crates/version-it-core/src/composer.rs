use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::blocks::{VersionTemplate, VersionBlock, BlockType, VersionContext};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionComposer {
    pub templates: HashMap<String, VersionTemplate>,
    pub counters: HashMap<String, u32>,
    pub default_template: Option<String>,
}

impl VersionComposer {
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
            counters: HashMap::new(),
            default_template: None,
        }
    }

    pub fn with_template(mut self, template: VersionTemplate) -> Self {
        self.templates.insert(template.name.clone(), template);
        self
    }

    pub fn with_default_template(mut self, template_name: &str) -> Self {
        self.default_template = Some(template_name.to_string());
        self
    }

    pub fn add_template(&mut self, template: VersionTemplate) {
        self.templates.insert(template.name.clone(), template);
    }

    pub fn get_template(&self, name: &str) -> Option<&VersionTemplate> {
        self.templates.get(name)
    }

    pub fn set_counter(&mut self, name: &str, value: u32) {
        self.counters.insert(name.to_string(), value);
    }

    pub fn increment_counter(&mut self, name: &str) -> u32 {
        let current = self.counters.get(name).copied().unwrap_or(0);
        let new_value = current + 1;
        self.counters.insert(name.to_string(), new_value);
        new_value
    }

    pub fn generate_version(&mut self, template_name: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
        let template_name = template_name
            .or(self.default_template.as_deref())
            .ok_or("No template specified and no default template set")?;

        let template = self.templates.get(template_name)
            .ok_or(format!("Template '{}' not found", template_name))?;

        let mut context = self.build_context();
        template.generate(&mut context)
    }

    pub fn generate_version_with_context(
        &mut self,
        template_name: Option<&str>,
        context: VersionContext,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let template_name = template_name
            .or(self.default_template.as_deref())
            .ok_or("No template specified and no default template set")?;

        let template = self.templates.get(template_name)
            .ok_or(format!("Template '{}' not found", template_name))?;

        let mut final_context = self.build_context();

        // Merge provided context
        if let Some(version) = context.current_version {
            final_context.current_version = Some(version);
        }
        if let Some(commit) = context.current_commit {
            final_context.current_commit = Some(commit);
        }
        if let Some(branch) = context.current_branch {
            final_context.current_branch = Some(branch);
        }
        if let Some(build) = context.build_number {
            final_context.build_number = Some(build);
        }
        for (name, value) in context.counters {
            final_context.counters.insert(name, value);
        }

        template.generate(&mut final_context)
    }

    fn build_context(&self) -> VersionContext {
        let mut context = VersionContext::new();

        // Set counters
        for (name, value) in &self.counters {
            context = context.with_counter(name, *value);
        }

        // Try to get current git info
        if let Ok(output) = Command::new("git").args(["rev-parse", "--short", "HEAD"]).output() {
            if output.status.success() {
                let commit = String::from_utf8_lossy(&output.stdout).trim().to_string();
                context.current_commit = Some(commit);
            }
        }

        if let Ok(output) = Command::new("git").args(["rev-parse", "--abbrev-ref", "HEAD"]).output() {
            if output.status.success() {
                let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
                context.current_branch = Some(branch);
            }
        }

        context
    }

    pub fn list_templates(&self) -> Vec<&String> {
        self.templates.keys().collect()
    }

    pub fn from_config(config: &ComposerConfig) -> Self {
        let mut composer = Self::new();

        // Set default template
        if let Some(ref default) = config.default_template {
            composer.default_template = Some(default.clone());
        }

        // Add counters
        for (name, value) in &config.counters {
            composer.set_counter(name, *value);
        }

        // Add templates
        for template in &config.templates {
            composer.add_template(template.clone());
        }

        composer
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposerConfig {
    pub default_template: Option<String>,
    pub counters: HashMap<String, u32>,
    pub templates: Vec<VersionTemplate>,
}

impl ComposerConfig {
    pub fn new() -> Self {
        Self {
            default_template: None,
            counters: HashMap::new(),
            templates: Vec::new(),
        }
    }

    pub fn from_yaml(yaml: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config: ComposerConfig = serde_yaml::from_str(yaml)?;
        Ok(config)
    }

    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = std::fs::read_to_string(path)?;
        Self::from_yaml(&contents)
    }

    pub fn add_template(&mut self, template: VersionTemplate) {
        self.templates.push(template);
    }

    pub fn set_counter(&mut self, name: &str, value: u32) {
        self.counters.insert(name.to_string(), value);
    }

    pub fn set_default_template(&mut self, template_name: &str) {
        self.default_template = Some(template_name.to_string());
    }
}

// Builder functions for common version templates
impl VersionTemplate {
    pub fn semantic_version() -> Self {
        Self::new("semantic")
            .add_block(
                VersionBlock::new("version", BlockType::Semantic { major: None, minor: None, patch: None })
            )
    }

    pub fn calver_short() -> Self {
        Self::new("calver-short")
            .add_block(
                VersionBlock::new("date", BlockType::Calver { year: None, month: None, day: None })
                    .with_format("YY.MM.DD")
            )
    }

    pub fn calver_long() -> Self {
        Self::new("calver-long")
            .add_block(
                VersionBlock::new("date", BlockType::Calver { year: None, month: None, day: None })
                    .with_format("YYYY.MM.DD")
            )
    }

    pub fn timestamped() -> Self {
        Self::new("timestamped")
            .with_separator("-")
            .add_block(
                VersionBlock::new("version", BlockType::Semantic { major: None, minor: None, patch: None })
            )
            .add_block(
                VersionBlock::new("timestamp", BlockType::Timestamp)
            )
    }

    pub fn commit_based() -> Self {
        Self::new("commit-based")
            .with_separator("-")
            .add_block(
                VersionBlock::new("version", BlockType::Semantic { major: None, minor: None, patch: None })
            )
            .add_block(
                VersionBlock::new("commit", BlockType::Commit)
            )
    }

    pub fn build_numbered() -> Self {
        Self::new("build-numbered")
            .with_separator("-")
            .add_block(
                VersionBlock::new("version", BlockType::Semantic { major: None, minor: None, patch: None })
            )
            .add_block(
                VersionBlock::new("build", BlockType::BuildNumber)
            )
    }

    pub fn custom(name: &str) -> Self {
        Self::new(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_composer_basic() {
        let template = VersionTemplate::semantic_version();
        let mut composer = VersionComposer::new().with_template(template);

        let result = composer.generate_version(Some("semantic"));
        assert!(result.is_ok());
        let version = result.unwrap();
        assert!(semver::Version::parse(&version).is_ok());
    }

    #[test]
    fn test_version_composer_with_counter() {
        let template = VersionTemplate::new("test")
            .with_separator("-")
            .add_block(
                VersionBlock::new("version", BlockType::Semantic { major: Some(1), minor: Some(0), patch: Some(0) })
            )
            .add_block(
                VersionBlock::new("counter", BlockType::Counter { name: "build".to_string() })
            );

        let mut composer = VersionComposer::new()
            .with_template(template)
            .with_default_template("test");

        composer.set_counter("build", 5);
        let result = composer.generate_version(None);
        assert_eq!(result.unwrap(), "1.0.0-5");
    }

    #[test]
    fn test_increment_counter() {
        let mut composer = VersionComposer::new();
        assert_eq!(composer.increment_counter("test"), 1);
        assert_eq!(composer.increment_counter("test"), 2);
        assert_eq!(composer.increment_counter("other"), 1);
    }

    #[test]
    fn test_composer_config() {
        let mut config = ComposerConfig::new();
        config.set_counter("build", 10);
        config.set_default_template("semantic");

        let template = VersionTemplate::semantic_version();
        config.add_template(template);

        let composer = VersionComposer::from_config(&config);
        assert_eq!(composer.counters.get("build"), Some(&10));
        assert_eq!(composer.default_template.as_deref(), Some("semantic"));
        assert!(composer.templates.contains_key("semantic"));
    }

    #[test]
    fn test_preset_templates() {
        let semantic = VersionTemplate::semantic_version();
        assert_eq!(semantic.name, "semantic");
        assert_eq!(semantic.blocks.len(), 1);

        let calver = VersionTemplate::calver_short();
        assert_eq!(calver.name, "calver-short");
        assert_eq!(calver.blocks.len(), 1);

        let timestamped = VersionTemplate::timestamped();
        assert_eq!(timestamped.name, "timestamped");
        assert_eq!(timestamped.blocks.len(), 2);
        assert_eq!(timestamped.separator, "-");
    }

    #[test]
    fn test_complex_template() {
        let template = VersionTemplate::custom("release")
            .with_prefix("v")
            .with_separator("-")
            .add_block(
                VersionBlock::new("major", BlockType::Semantic { major: Some(2), minor: Some(0), patch: Some(0) })
            )
            .add_block(
                VersionBlock::new("date", BlockType::Date { format: "%Y%m%d".to_string() })
            )
            .add_block(
                VersionBlock::new("branch", BlockType::Branch)
            )
            .add_block(
                VersionBlock::new("build", BlockType::Counter { name: "release".to_string() })
            );

        let mut composer = VersionComposer::new().with_template(template);
        composer.set_counter("release", 3);

        let result = composer.generate_version(Some("release"));
        assert!(result.is_ok());
        let version = result.unwrap();
        assert!(version.starts_with("v2.0.0-"));
        assert!(version.ends_with("-3"));
    }
}