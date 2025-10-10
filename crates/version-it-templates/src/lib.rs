use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;
use version_it_config::VersionHeader;

pub trait TemplateManager {
    fn generate_headers(&self, headers: &[VersionHeader], version: &str, channel: Option<&str>, base_path: &Path) -> Result<(), Box<dyn std::error::Error>>;
}

pub struct DefaultTemplateManager;

impl TemplateManager for DefaultTemplateManager {
    fn generate_headers(&self, headers: &[VersionHeader], version: &str, channel: Option<&str>, base_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        for header in headers {
            self.generate_single_header(header, version, channel, base_path)?;
        }
        Ok(())
    }
}

impl DefaultTemplateManager {
    fn generate_single_header(&self, header: &VersionHeader, version: &str, channel: Option<&str>, base_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let handlebars = handlebars::Handlebars::new();
        let template = if let Some(ref template_path) = header.template_path {
            let full_template_path = base_path.join(template_path);
            std::fs::read_to_string(full_template_path)?
        } else if let Some(ref template) = header.template {
            template.clone()
        } else {
            return Err("Either template or template-path must be specified for version header".into());
        };

        let git_info = self.gather_git_info();
        let project_info = self.gather_project_info();
        let stats_info = serde_json::json!({
            "file_count": "disabled",
            "lines_of_code": "disabled"
        });

        let data = serde_json::json!({
            "version": version,
            "scheme": "semantic", // This would need to be passed in
            "channel": channel.unwrap_or(""),
            "git": git_info,
            "build": {
                "timestamp": Self::current_datetime(),
                "date": Self::build_date(),
                "time": Self::build_time(),
                "compiler": Self::rustc_version()
            },
            "system": {
                "hostname": Self::hostname(),
                "username": Self::username(),
                "os": Self::os_info(),
                "arch": Self::arch_info(),
                "cpus": Self::cpu_count(),
                "memory": Self::available_memory()
            },
            "project": project_info,
            "stats": stats_info
        });

        let content = handlebars.render_template(&template, &data)?;
        let full_path = base_path.join(&header.path);
        std::fs::write(&full_path, content)?;
        Ok(())
    }

    fn current_datetime() -> String {
        let now = chrono::Utc::now();
        now.format("%Y-%m-%dT%H:%M:%S").to_string()
    }

    fn build_date() -> String {
        let now = chrono::Utc::now();
        now.format("%Y-%m-%d").to_string()
    }

    fn build_time() -> String {
        let now = chrono::Utc::now();
        now.format("%H:%M:%S").to_string()
    }

    fn rustc_version() -> String {
        "unknown".to_string()
    }

    fn os_info() -> String {
        "unknown unknown".to_string()
    }

    fn arch_info() -> String {
        "unknown".to_string()
    }

    fn cpu_count() -> usize {
        num_cpus::get()
    }

    fn available_memory() -> u64 {
        sysinfo::System::new_all().available_memory()
    }

    fn hostname() -> String {
        hostname::get()
            .ok()
            .and_then(|h| h.to_str().map(|s| s.to_string()))
            .unwrap_or_else(|| "unknown".to_string())
    }

    fn username() -> String {
        std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| "unknown".to_string())
    }

    fn gather_git_info(&self) -> serde_json::Value {
        serde_json::json!({
            "commit_hash": self.current_commit_full(),
            "commit_hash_short": self.current_commit_short(),
            "branch": self.current_branch(),
            "tags": self.current_tags()
        })
    }

    fn gather_project_info(&self) -> serde_json::Value {
        serde_json::json!({
            "name": "unknown",
            "version": "unknown",
            "authors": "unknown",
            "description": "unknown"
        })
    }

    fn current_commit_full(&self) -> String {
        Command::new("git")
            .args(["rev-parse", "HEAD"])
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "unknown".to_string())
    }

    fn current_commit_short(&self) -> String {
        Command::new("git")
            .args(["rev-parse", "--short", "HEAD"])
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "unknown".to_string())
    }

    fn current_branch(&self) -> String {
        Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "unknown".to_string())
    }

    fn current_tags(&self) -> Vec<String> {
        Command::new("git")
            .args(["tag", "--points-at", "HEAD"])
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .map(|s| s.lines().map(|l| l.to_string()).collect())
            .unwrap_or_default()
    }
}