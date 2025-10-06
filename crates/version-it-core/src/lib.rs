pub mod version;
pub mod config;
pub mod git;
pub mod templates;
pub mod package;
pub mod utils;

// Re-export public items
pub use version::{VersionInfo, VersionType};
pub use config::{Config, ChangelogExporters, ChangelogSection, ChangeSubstitution, ChangeAction, ChangeTypeMap, VersionHeader, PackageFile};