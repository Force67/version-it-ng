use handlebars::Handlebars;
use serde_json;
use chrono::{DateTime, Utc};
use toml;

impl super::Config {
    fn current_datetime() -> String {
        let now: DateTime<Utc> = Utc::now();
        now.format("%Y-%m-%dT%H:%M:%S").to_string()
    }

    fn build_date() -> String {
        let now: DateTime<Utc> = Utc::now();
        now.format("%Y-%m-%d").to_string()
    }

    fn build_time() -> String {
        let now: DateTime<Utc> = Utc::now();
        now.format("%H:%M:%S").to_string()
    }

    fn hostname() -> String {
        std::env::var("HOSTNAME").unwrap_or_else(|_| "unknown".to_string())
    }

    fn username() -> String {
        std::env::var("USER").or_else(|_| std::env::var("USERNAME")).unwrap_or_else(|_| "unknown".to_string())
    }

    fn gather_project_info() -> serde_json::Value {
        // Try to read Cargo.toml
        let mut name = "unknown".to_string();
        let mut description = "unknown".to_string();
        let mut authors = vec![];

        if let Ok(content) = std::fs::read_to_string("Cargo.toml") {
            if let Ok(toml) = toml::from_str::<toml::Value>(&content) {
                if let Some(package) = toml.get("package") {
                    if let Some(n) = package.get("name") {
                        name = n.as_str().unwrap_or("unknown").to_string();
                    }
                    if let Some(d) = package.get("description") {
                        description = d.as_str().unwrap_or("unknown").to_string();
                    }
                    if let Some(a) = package.get("authors") {
                        if let Some(arr) = a.as_array() {
                            authors = arr.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect();
                        }
                    }
                }
            }
        }

        serde_json::json!({
            "name": name,
            "description": description,
            "authors": authors
        })
    }

    fn gather_stats(&self) -> serde_json::Value {
        // Check for cached stats first
        let cache_file = ".version-it-stats-cache.json";
        if let Ok(_metadata) = std::fs::metadata(cache_file) {
            if let Ok(cache_content) = std::fs::read_to_string(cache_file) {
                if let Ok(cache) = serde_json::from_str::<serde_json::Value>(&cache_content) {
                    // Check if cache is still valid (within last hour)
                    if let Some(timestamp) = cache.get("timestamp").and_then(|t| t.as_u64()) {
                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();
                        if now - timestamp < 3600 { // 1 hour cache
                            return cache;
                        }
                    }
                }
            }
        }

        // Calculate stats (expensive operation)
        println!("Calculating project statistics... (this may take a moment)");
        let file_count = walkdir::WalkDir::new(".")
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .count();

        // Approximate lines of code (very basic)
        let mut lines_of_code = 0;
        walkdir::WalkDir::new(".")
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| {
                let path = e.path().to_string_lossy();
                path.ends_with(".rs") || path.ends_with(".js") || path.ends_with(".ts") || path.ends_with(".py")
            })
            .for_each(|e| {
                if let Ok(content) = std::fs::read_to_string(e.path()) {
                    lines_of_code += content.lines().count();
                }
            });

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let stats = serde_json::json!({
            "file_count": file_count,
            "lines_of_code": lines_of_code,
            "timestamp": timestamp
        });

        // Cache the results
        let _ = std::fs::write(cache_file, serde_json::to_string_pretty(&stats).unwrap_or_default());

        stats
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
            let handlebars = Handlebars::new();
            for header in headers {
                let template = if let Some(ref template_path) = header.template_path {
                    std::fs::read_to_string(template_path)?
                } else if let Some(ref template) = header.template {
                    template.clone()
                } else {
                    return Err("Either template or template-path must be specified for version header".into());
                };
                let git_info = Self::gather_git_info();
                let project_info = Self::gather_project_info();
                let stats_info = if self.enable_expensive_metrics {
                    self.gather_stats()
                } else {
                    serde_json::json!({
                        "file_count": "disabled",
                        "lines_of_code": "disabled"
                    })
                };
                let data = serde_json::json!({
                    "version": version,
                    "scheme": self.versioning_scheme,
                    "channel": channel.unwrap_or(""),
                    "git": git_info,
                    "build": {
                        "timestamp": Self::current_datetime(),
                        "date": Self::build_date(),
                        "time": Self::build_time(),
                        "compiler": super::VersionInfo::rustc_version()
                    },
                    "system": {
                        "hostname": Self::hostname(),
                        "username": Self::username(),
                        "os": super::VersionInfo::os_info(),
                        "arch": super::VersionInfo::arch_info(),
                        "cpus": super::VersionInfo::cpu_count(),
                        "memory": super::VersionInfo::available_memory()
                    },
                    "project": project_info,
                    "stats": stats_info
                });
                let content = handlebars.render_template(&template, &data)?;
                std::fs::write(&header.path, content)?;
            }
        }
        Ok(())
    }
}