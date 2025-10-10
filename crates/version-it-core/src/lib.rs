// Re-export public items from extracted crates
pub use version_it_version::{VersionInfo, VersionType};
pub use version_it_config::{Config, ChangelogExporters, ChangelogSection, ChangeSubstitution, ChangeAction, ChangeTypeMap, VersionHeader, PackageFile};
pub use version_it_blocks::{VersionBlock, BlockType, VersionContext, VersionTemplate};
pub use version_it_composer::{VersionComposer, ComposerConfig};
pub use version_it_package::PackageManager;
pub use version_it_templates::TemplateManager;
pub use version_it_git::GitManager;