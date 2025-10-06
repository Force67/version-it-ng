use semver::{Version, Prerelease, BuildMetadata};
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
            channel,
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

    fn current_timestamp() -> String {
        let now: DateTime<Utc> = Utc::now();
        now.format("%Y%m%d%H%M%S").to_string()
    }

    pub(crate) fn current_commit() -> Result<String, Box<dyn std::error::Error>> {
        let output = Command::new("git").args(["rev-parse", "--short", "HEAD"]).output()?;
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

    pub fn rustc_version() -> String {
        // Try to get rustc version
        if let Ok(output) = Command::new("rustc").args(["--version"]).output() {
            if output.status.success() {
                return String::from_utf8_lossy(&output.stdout).trim().to_string();
            }
        }
        "unknown".to_string()
    }

    pub fn os_info() -> String {
        std::env::consts::OS.to_string()
    }

    pub fn arch_info() -> String {
        std::env::consts::ARCH.to_string()
    }

    pub fn available_memory() -> String {
        use sysinfo::System;
        let mut sys = System::new_all();
        sys.refresh_all();
        let total_memory = sys.total_memory();
        let available_memory = sys.available_memory();
        format!("{} MB total, {} MB available", total_memory / 1024 / 1024, available_memory / 1024 / 1024)
    }

    pub fn cpu_count() -> usize {
        num_cpus::get()
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

impl fmt::Display for VersionInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

        let version_str = if let Some(ref channel) = self.channel {
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
        };

        write!(f, "{}", version_str)
    }
}