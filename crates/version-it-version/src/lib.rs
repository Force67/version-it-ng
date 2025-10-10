use chrono::{DateTime, Utc};
use std::process::Command;
use std::fmt;

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
    SemanticCommit { major: u32, minor: u32, commit_count: u32 },
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
    /// * `scheme` - The versioning scheme to use.
    /// * `channel` - Optional channel information.
    ///
    /// # Returns
    ///
    /// A Result containing the VersionInfo or an error string.
    pub fn new(version: &str, scheme: &str, channel: Option<String>) -> Result<Self, String> {
        let version_type = match scheme {
            "semantic" => {
                let semver = semver::Version::parse(version)
                    .map_err(|e| format!("Invalid semantic version '{}': {}", version, e))?;
                VersionType::Semantic(Version {
                    major: semver.major as u32,
                    minor: semver.minor as u32,
                    patch: semver.patch as u32,
                })
            }
            "calver" => {
                let parts: Vec<&str> = version.split('.').collect();
                if parts.len() != 3 {
                    return Err(format!("Invalid calendar version '{}': expected format YY.MM.DD", version));
                }
                let year = parts[0].parse().map_err(|_| format!("Invalid year in calendar version: {}", parts[0]))?;
                let month = parts[1].parse().map_err(|_| format!("Invalid month in calendar version: {}", parts[1]))?;
                let day = parts[2].parse().map_err(|_| format!("Invalid day in calendar version: {}", parts[2]))?;
                VersionType::Calver { year, month, day }
            }
            "timestamp" => VersionType::Timestamp(version.to_string()),
            "commit" => VersionType::Commit(version.to_string()),
            "build" => {
                let parts: Vec<&str> = version.split('.').collect();
                if parts.len() != 4 {
                    return Err(format!("Invalid build version '{}': expected format major.minor.patch.build", version));
                }
                let major = parts[0].parse().map_err(|_| format!("Invalid major version: {}", parts[0]))?;
                let minor = parts[1].parse().map_err(|_| format!("Invalid minor version: {}", parts[1]))?;
                let patch = parts[2].parse().map_err(|_| format!("Invalid patch version: {}", parts[2]))?;
                let build = parts[3].parse().map_err(|_| format!("Invalid build version: {}", parts[3]))?;
                VersionType::Build { major, minor, patch, build }
            }
            "monotonic" => {
                let num = version.parse().map_err(|_| format!("Invalid monotonic version: {}", version))?;
                VersionType::Monotonic(num)
            }
            "datetime" => VersionType::Datetime(version.to_string()),
            "pattern" => VersionType::Pattern(version.to_string()),
            "semantic-commit" => {
                let parts: Vec<&str> = version.split('.').collect();
                if parts.len() != 3 {
                    return Err(format!("Invalid semantic-commit version '{}': expected format major.minor.commit", version));
                }
                let major = parts[0].parse().map_err(|_| format!("Invalid major version: {}", parts[0]))?;
                let minor = parts[1].parse().map_err(|_| format!("Invalid minor version: {}", parts[1]))?;
                let commit_count = parts[2].parse().map_err(|_| format!("Invalid commit count: {}", parts[2]))?;
                VersionType::SemanticCommit { major, minor, commit_count }
            }
            _ => return Err(format!("Unsupported versioning scheme: {}", scheme)),
        };

        Ok(VersionInfo {
            scheme: scheme.to_string(),
            version: version_type,
            channel,
        })
    }

    /// Returns the version as a string.
    pub fn to_string(&self) -> String {
        match &self.version {
            VersionType::Semantic(v) => format!("{}.{}.{}", v.major, v.minor, v.patch),
            VersionType::Calver { year, month, day } => format!("{:02}.{:02}.{:02}", year, month, day),
            VersionType::Timestamp(s) => s.clone(),
            VersionType::Commit(s) => s.clone(),
            VersionType::Build { major, minor, patch, build } => format!("{}.{}.{}.{}", major, minor, patch, build),
            VersionType::Monotonic(n) => n.to_string(),
            VersionType::Datetime(s) => s.clone(),
            VersionType::Pattern(s) => s.clone(),
            VersionType::SemanticCommit { major, minor, commit_count } => format!("{}.{}.{}", major, minor, commit_count),
        }
    }

    /// Bumps the major version.
    pub fn bump_major(&mut self) {
        match &mut self.version {
            VersionType::Semantic(v) => {
                v.major += 1;
                v.minor = 0;
                v.patch = 0;
            }
            VersionType::Calver { year, .. } => {
                *year += 1;
            }
            VersionType::Build { major, .. } => {
                *major += 1;
            }
            VersionType::SemanticCommit { major, .. } => {
                *major += 1;
            }
            _ => {} // Other schemes don't support major bumps
        }
    }

    /// Bumps the minor version.
    pub fn bump_minor(&mut self) {
        match &mut self.version {
            VersionType::Semantic(v) => {
                v.minor += 1;
                v.patch = 0;
            }
            VersionType::Calver { month, day, .. } => {
                *month += 1;
                *day = 1;
            }
            VersionType::Build { minor, build, .. } => {
                *minor += 1;
                *build = 0;
            }
            VersionType::SemanticCommit { minor, commit_count, .. } => {
                *minor += 1;
                *commit_count = 0;
            }
            _ => {} // Other schemes don't support minor bumps
        }
    }

    /// Bumps the patch version.
    pub fn bump_patch(&mut self) {
        match &mut self.version {
            VersionType::Semantic(v) => {
                v.patch += 1;
            }
            VersionType::Calver { day, .. } => {
                *day += 1;
            }
            VersionType::Build { patch, build, .. } => {
                *patch += 1;
                *build = 0;
            }
            VersionType::Timestamp(_) => {
                self.version = VersionType::Timestamp(Self::current_timestamp());
            }
            VersionType::Commit(_) => {
                self.version = VersionType::Commit(Self::current_commit_short());
            }
            VersionType::SemanticCommit { commit_count, .. } => {
                *commit_count += 1;
            }
            _ => {} // Other schemes don't support patch bumps
        }
    }

    fn current_timestamp() -> String {
        let now: DateTime<Utc> = Utc::now();
        now.format("%Y%m%d%H%M%S").to_string()
    }

    fn current_commit_short() -> String {
        Command::new("git")
            .args(["rev-parse", "--short", "HEAD"])
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "unknown".to_string())
    }

    /// Gets system information for templates
    pub fn rustc_version() -> String {
        "unknown".to_string()
    }

    pub fn os_info() -> String {
        "unknown unknown".to_string()
    }

    pub fn arch_info() -> String {
        "unknown".to_string()
    }

    pub fn cpu_count() -> usize {
        num_cpus::get()
    }

    pub fn available_memory() -> u64 {
        sysinfo::System::new_all().available_memory()
    }
}

#[derive(Debug, Clone)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl fmt::Display for VersionInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}