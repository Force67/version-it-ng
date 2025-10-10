# version-it

A powerful versioning tool for CI pipelines that supports multiple versioning schemes and **custom version crafting** with configurable building blocks.

## ✨ New: Custom Version Crafting

Create **arbitrary version strings** by combining different building blocks in any order you want! Mix semantic numbers, timestamps, git commits, counters, and more to craft perfect version schemes for your project.

```bash
# List available version templates
version-it craft --list-templates

# Generate version using custom template
version-it craft --template enterprise-release

# Increment build counters
version-it craft --increment-counter build

# Set counter values
version-it craft --set-counter release:10
```

## Versioning Schemes

- **semantic**: Standard semantic versioning (1.2.3)
- **calver**: Calendar versioning (25.10.01 for Oct 1, 2025)
- **timestamp**: Timestamp-based (20251005220904)
- **commit**: Git commit hash-based (abc1234)
- **build**: Build number versioning (1.2.3.456)
- **monotonic**: Simple incrementing number (42)
- **datetime**: ISO 8601 datetime (2024-10-06T14:30:00)
- **pattern**: Custom string patterns (v1.0.0-snapshot)
- **semantic-commit**: Semantic versioning with commit count (1.23.456)
- **🎯 custom**: Craft your own versions with building blocks!

## Release Channels

Support for release channels with different versioning behaviors:

- **stable**: Standard versioning (1.2.3)
- **beta**: Pre-release versions (1.2.3-beta.1)
- **nightly**: Uses timestamp/commit for nightly builds (20241006)
- **custom**: User-defined channel suffix (1.2.3-custom)

## Bump Types

Version-it supports three standard bump types that work differently depending on your versioning scheme:

### **patch** - Small Changes
- **Semantic**: Increments patch number (1.2.3 → 1.2.4)
- **Calendar**: Increments day (25.10.15 → 25.10.16)
- **Build**: Increments build number (1.2.3.456 → 1.2.3.457)
- **Timestamp**: Updates to current timestamp
- **Use for**: Bug fixes, documentation, internal improvements

### **minor** - New Features
- **Semantic**: Increments minor, resets patch (1.2.3 → 1.3.0)
- **Calendar**: Increments month, resets day (25.10.15 → 25.11.01)
- **Build**: Increments minor, resets build (1.2.3.456 → 1.2.4.0)
- **Use for**: New features, backwards-compatible changes

### **major** - Breaking Changes
- **Semantic**: Increments major, resets minor/patch (1.2.3 → 2.0.0)
- **Calendar**: Increments year, resets month/day (25.10.15 → 26.01.01)
- **Build**: Increments major, resets minor/build (1.2.3.456 → 1.3.0.0)
- **Use for**: Breaking changes, major rewrites

💡 **Pro tip**: Use `version-it next --bump <type>` to preview what a bump would produce!

## Usage

```bash
# Semantic versioning
version-it bump --version 1.0.0 --bump patch
# Output: 1.0.1

# Semantic-commit versioning
version-it bump --version 1.23.456 --bump minor
# Output: 1.24.<current_commit_count>

# Calendar versioning
version-it bump --version 25.10.01 --bump minor
# Output: 25.11.01

# Timestamp versioning (uses current time)
version-it bump --bump patch
# Output: 20251005220904

# Commit versioning (uses current git commit)
version-it bump --bump patch
# Output: abc1234

# Build versioning
version-it bump --version 1.2.3.456 --bump patch
# Output: 1.2.4.0

# Monotonic versioning
version-it bump --version 42 --bump major
# Output: 43

# Datetime versioning (uses current datetime)
version-it bump --bump patch
# Output: 2024-10-06T14:30:00

# Pattern versioning
version-it bump --version v1.0.0-snapshot --bump minor
# Output: v1.0.0-snapshot-updated

# Override scheme via CLI
version-it bump --version 1.2.3 --scheme build --bump patch
# Output: 1.2.4.0

version-it bump --version 1.23.456 --scheme semantic-commit --bump major
# Output: 2.0.<current_commit_count>

# Channel-based versioning
version-it bump --version 1.2.3 --channel beta --bump patch
# Output: 1.2.4-beta.1

version-it bump --version 1.2.3 --channel nightly --scheme timestamp --bump patch
# Output: 20251005220904

version-it bump --version 1.2.3 --channel rc --bump minor
# Output: 1.3.0-rc

# Automatically bump based on commits
version-it auto-bump
# Analyzes git commits since last version tag and bumps accordingly (when enabled)

# Bump with git operations
version-it bump --version 1.0.0 --bump minor --commit --create-tag
# Bumps version, commits changes, and creates annotated git tag

version-it auto-bump --commit --create-tag
# Auto-bump with automatic commit and tag creation

# Dry-run mode (preview changes without applying them)
version-it bump --version 1.0.0 --bump minor --dry-run
# Shows what would happen without making actual changes

version-it auto-bump --dry-run --commit --create-tag
# Shows auto-bump operations that would be performed
```

## 🎯 Version Crafting

### Available Building Blocks

- **semantic**: Traditional semantic versioning (major.minor.patch)
- **calver**: Calendar versioning with flexible formats (YY.MM.DD, YYYYMMDD, etc.)
- **timestamp**: Various timestamp formats (unix, YYYYMMDDHHMMSS, iso, etc.)
- **commit**: Git commit hash (short or full)
- **counter**: Incremental counters for builds/releases
- **text**: Static text values
- **date**: Formatted dates with custom patterns
- **branch**: Git branch name
- **build_number**: Build number from context
- **versioned**: Reference to another block's value

### Configuration

Create a `version-templates.yaml` file to define your custom version templates:

```yaml
default_template: "enterprise-release"

counters:
  build: 42
  release: 5

templates:
  # Enterprise release: v3.0.0-2025.10.10-abc123-42
  - name: "enterprise-release"
    prefix: "v"
    separator: "-"
    blocks:
      - name: "version"
        type: "semantic"
        config:
          major: 3
          minor: 0
          patch: 0
      - name: "date"
        type: "calver"
        format: "YYYY.MM.DD"
      - name: "commit"
        type: "commit"
      - name: "build"
        type: "counter"
        config:
          name: "build"

  # Feature branch: 1.5.0-feature-branch-7
  - name: "feature-branch"
    separator: "-"
    blocks:
      - name: "base"
        type: "semantic"
        config:
          major: 1
          minor: 5
          patch: 0
      - name: "branch"
        type: "branch"
      - name: "build"
        type: "counter"
        config:
          name: "build"

  # Timestamped release: release-2.1.0-20251010-abc123
  - name: "timestamped-release"
    prefix: "release-"
    separator: "-"
    blocks:
      - name: "version"
        type: "semantic"
        config:
          major: 2
          minor: 1
          patch: 0
      - name: "timestamp"
        type: "timestamp"
        format: "YYYYMMDD"
      - name: "commit"
        type: "commit"

  # Simple semantic: v1.2.3
  - name: "semantic"
    prefix: "v"
    blocks:
      - name: "version"
        type: "semantic"

  # Calendar version: 25.10.10
  - name: "calver"
    blocks:
      - name: "date"
        type: "calver"
        format: "YY.MM.DD"
```

### Craft Command Examples

```bash
# Generate version using default template
version-it craft

# Generate version using specific template
version-it craft --template enterprise-release

# Use custom configuration file
version-it craft --config-file custom-templates.yaml

# List all available templates
version-it craft --list-templates

# Increment counters
version-it craft --increment-counter build
version-it craft --increment-counter release

# Set counter to specific value
version-it craft --set-counter build:100
version-it craft --set-counter release:5

# Preview what would happen (dry run)
version-it craft --template enterprise-release --dry-run

# Generate structured JSON output
version-it craft --template enterprise-release --structured-output
```

### Block Formats

#### Calver Formats
- `YY.MM.DD` → 25.10.10
- `YYYY.MM.DD` → 2025.10.10
- `YYMMDD` → 251010
- `YYYYMMDD` → 20251010

#### Timestamp Formats
- `unix` → 1728524400
- `unix_ms` → 1728524400123
- `YYYYMMDDHHMMSS` → 20251010143000
- `iso` → 2025-10-10T14:30:00Z

#### Date Formats
- `%Y-%m-%d` → 2025-10-10
- `%d/%m/%Y` → 10/10/2025
- Any chrono-compatible format

## Configuration

Most options can be set via YAML config or overridden via CLI flags.

Specify a custom config file with `--config path/to/.version-it`.

Create a `.version-it` file in your project:

```yaml
versioning-scheme: calver
first-version: 25.10.01
channel: stable  # Optional: release channel (stable, beta, nightly, or custom)
current-version-file: version.txt  # Optional: read/write current version from/to this file
commit-based-bumping: true  # Optional: enable automatic bumping based on commit messages
enable-expensive-metrics: true  # Optional: enable expensive metrics (file/line counting) - cached for 1 hour
version-headers:
- path: include/version.h
   template: |
     #ifndef VERSION_H
     #define VERSION_H
     #define VERSION "{{version}}"
     #define SCHEME "{{scheme}}"
     #define CHANNEL "{{channel}}"
     #define GIT_COMMIT "{{git.commit_hash}}"
     #define GIT_BRANCH "{{git.branch}}"
     #define BUILD_DATE "{{build.date}}"
     #define PROJECT_NAME "{{project.name}}"
     #define COMMIT_COUNT {{git.commit_count}}
     #endif

package-files:
- path: package.json
  manager: npm
- path: Cargo.toml
  manager: cargo
```

## Templates

Templates use Handlebars syntax and are completely language-independent. Available variables:

**Version Information:**
- `{{version}}`: The current version string
- `{{scheme}}`: The versioning scheme (semantic, calver, etc.)
- `{{channel}}`: The release channel (stable, beta, nightly, etc.)

**Git Information:**
- `{{git.commit_hash}}`: Short git commit hash
- `{{git.commit_hash_full}}`: Full git commit hash
- `{{git.branch}}`: Current git branch name
- `{{git.tag}}`: Latest git tag (empty if none)
- `{{git.author}}`: Commit author name
- `{{git.email}}`: Commit author email
- `{{git.date}}`: Commit date (ISO 8601 format)
- `{{git.commit_count}}`: Total number of commits
- `{{git.first_commit_date}}`: Date of first commit
- `{{git.recent_commits}}`: Array of recent commits (last 10)

**Build Information:**
- `{{build.timestamp}}`: Build timestamp (ISO 8601 format)
- `{{build.date}}`: Build date (YYYY-MM-DD)
- `{{build.time}}`: Build time (HH:MM:SS)
- `{{build.compiler}}`: Compiler version (rustc version)

**System Information:**
- `{{system.hostname}}`: System hostname
- `{{system.username}}`: Current user name
- `{{system.os}}`: Operating system
- `{{system.arch}}`: System architecture
- `{{system.cpus}}`: Number of CPU cores
- `{{system.memory}}`: System memory (total and available in MB)

**Project Information:**
- `{{project.name}}`: Project name (from Cargo.toml)
- `{{project.description}}`: Project description
- `{{project.authors}}`: Array of project authors

**Statistics:**
- `{{stats.file_count}}`: Total number of files in project
- `{{stats.lines_of_code}}`: Approximate lines of code

You can specify templates inline with the `template` field or reference external template files with `template-path`.

See `examples/templates/` for sample templates.

## Package Files

The tool can automatically update version fields in package manager files:

- **npm**: Updates `package.json` version field
- **cargo**: Updates `Cargo.toml` version field
- **python**: Updates `__version__` in Python files
- **maven**: Updates `<version>` tags in `pom.xml`
- **cmake**: Updates `set(PROJECT_VERSION ...)` in CMakeLists.txt
- **cmake**: Updates `set(PROJECT_VERSION ...)` in CMakeLists.txt

Configure package files in your `.version-it` config:

```yaml
package-files:
- path: package.json
  manager: npm
- path: Cargo.toml
  manager: cargo
- path: CMakeLists.txt
  manager: cmake
  field: PROJECT_VERSION  # Optional: specify variable name
- path: pyproject.toml
  manager: python
  field: __version__  # Optional: specify field name
```

## Development

Requires Rust toolchain. Build and test:

```bash
cargo build
cargo test
```

## Subproject Support

For monorepos with multiple subprojects, create separate `.version-it` configs in each subfolder. Use `current-version-file` to store the version for each subproject independently of global git tags.

Example:
```
repo/
├── .version-it          # Global config
├── subproject1/
│   ├── .version-it      # Subproject config with current-version-file: version.txt
│   └── version.txt      # Current version for subproject1
└── subproject2/
    └── .version-it      # Subproject config
```

Run commands with `--config subproject1/.version-it` to work on specific subprojects.

## Commit-Based Bumping

Enable automatic version bumping based on commit messages by setting `commit-based-bumping: true` in your config. Define patterns that map to version bump types:

```yaml
commit-based-bumping: true
change-type-map:
  - label: "feat"
    action: minor
  - label: "fix"
    action: patch
  - label: "BREAKING"
    action: major
  # Regex patterns for more complex matching
  - pattern: "feat.*\\(major\\)"
    action: major
  - pattern: "fix.*security"
    action: minor
```

Available actions: `patch`, `minor`, `major`, `null` (ignore)

## CI Integration

When `commit-based-bumping` is enabled, the `auto-bump` command analyzes git commits since the last version tag and determines the appropriate bump based on configured labels:

- Checks if current branch is in `run-on-branches`
- Finds commits since last version tag
- Matches commit messages against `change-type-map` labels
- Applies the highest priority bump (patch < minor < major)
- Generates updated header files

Example CI workflow:
```yaml
- name: Auto bump version
  run: |
    NEW_VERSION=$(version-it auto-bump)
    echo "version=$NEW_VERSION" >> $GITHUB_OUTPUT
```

## 🚀 CLI Commands

### version-it craft
Generate custom versions using configurable templates.

```bash
# Generate version using default template
version-it craft

# Use specific template
version-it craft --template enterprise-release

# List available templates
version-it craft --list-templates

# Manage counters
version-it craft --increment-counter build
version-it craft --set-counter release:10

# Use custom config file
version-it craft --config-file my-templates.yaml

# Preview operations
version-it craft --template enterprise-release --dry-run
```

### version-it bump
Bump version components (traditional commands still work).

```bash
version-it bump --version 1.0.0 --bump patch
version-it bump --version 1.2.3 --scheme semantic --bump minor
```

### version-it auto-bump
Automatically bump based on commit messages.

```bash
version-it auto-bump --commit --create-tag
```

### version-it next
Preview next version without bumping.

```bash
version-it next --version 1.0.0 --bump minor
```

### version-it monorepo
Process multiple subprojects in a monorepo with a single command.

```bash
# Preview monorepo bump
version-it monorepo --bump patch --dry-run

# Bump all subprojects at once
version-it monorepo --bump minor

# Full monorepo release
version-it monorepo --bump major --commit --create-tag
```

Configure subprojects in your root `.version-it`:

```yaml
subprojects:
  - path: packages/component1
  - path: packages/component2
    config: packages/component2/custom-config.yml
```

## 🔧 Advanced Examples

### Complex Enterprise Versioning
```yaml
# Generates: release-myapp-4.2.1-20251010-main-7-GA
- name: "enterprise"
  prefix: "release-"
  separator: "-"
  suffix: "-GA"
  blocks:
    - name: "product"
      type: "text"
      config:
        value: "myapp"
    - name: "version"
      type: "semantic"
      config:
        major: 4
        minor: 2
        patch: 1
    - name: "date"
      type: "calver"
      format: "YYYYMMDD"
    - name: "branch"
      type: "branch"
    - name: "build"
      type: "counter"
      config:
        name: "release"
```

### Multi-Component Feature Release
```yaml
# Generates: v3.0.0-25.10-12-abc123
- name: "feature-release"
  prefix: "v"
  separator: "-"
  blocks:
    - name: "semantic"
      type: "semantic"
      config:
        major: 3
        minor: 0
        patch: 0
    - name: "date"
      type: "calver"
      format: "YY.MM"
    - name: "release"
      type: "counter"
      config:
        name: "release"
    - name: "commit"
      type: "commit"
```

### Monorepo Versioning with References
```yaml
# Generates: 1.2.3.1.2.3.5
- name: "monorepo-ref"
  blocks:
    - name: "base"
      type: "semantic"
      config:
        major: 1
        minor: 2
        patch: 3
    - name: "ref"
      type: "versioned"
      config:
        name: "base"
    - name: "build"
      type: "counter"
      config:
        name: "build"
```

## 📁 Examples

- `examples/version-templates.yaml` - Comprehensive template examples
- `examples/simple-templates.yaml` - Simple starter templates
- `examples/demo.rs` - Rust API usage demo

## 🎯 Why Version Crafting?

Traditional versioning schemes are rigid, but real projects often need custom formats:

- **Enterprise releases** need product names, build numbers, and GA suffixes
- **Feature branches** need branch names and iteration counts
- **Monorepos** need to reference multiple version components
- **Compliance** requires specific date formats and audit trails
- **DevOps** workflows need build server integration

With version crafting, you can create exactly the version format you need by combining building blocks in any order you want! 🚀
