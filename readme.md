# version-it

A configurable versioning tool for CI pipelines supporting multiple versioning schemes.

## Versioning Schemes

- **semantic**: Standard semantic versioning (1.2.3)
- **calver**: Calendar versioning (25.10.01 for Oct 1, 2025)
- **timestamp**: Timestamp-based (20251005220904)
- **commit**: Git commit hash-based (abc1234)
- **build**: Build number versioning (1.2.3.456)
- **monotonic**: Simple incrementing number (42)
- **datetime**: ISO 8601 datetime (2024-10-06T14:30:00)
- **pattern**: Custom string patterns (v1.0.0-snapshot)

## Usage

```bash
# Semantic versioning
version-it bump --version 1.0.0 --bump patch
# Output: 1.0.1

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

# Automatically bump based on commits
version-it auto-bump
# Analyzes git commits since last version tag and bumps accordingly

# Bump with git operations
version-it bump --version 1.0.0 --bump minor --commit --create-tag
# Bumps version, commits changes, and creates annotated git tag

version-it auto-bump --commit --create-tag
# Auto-bump with automatic commit and tag creation
```

## Configuration

Most options can be set via YAML config or overridden via CLI flags.

Specify a custom config file with `--config path/to/.version-it`.

Create a `.version-it` file in your project:

```yaml
versioning-scheme: calver
first-version: 25.10.01
current-version-file: version.txt  # Optional: read/write current version from/to this file
version-headers:
- path: include/version.h
  template: |
    #ifndef VERSION_H
    #define VERSION_H
    #define VERSION "{{version}}"
    #endif

package-files:
- path: package.json
  manager: npm
- path: Cargo.toml
  manager: cargo
```

## Templates

Templates use Handlebars syntax and are completely language-independent. Available variables:
- `{{version}}`: The current version string
- `{{scheme}}`: The versioning scheme (semantic, calver, etc.)

You can specify templates inline with the `template` field or reference external template files with `template-path`.

See `examples/templates/` for sample templates.

## Package Files

The tool can automatically update version fields in package manager files:

- **npm**: Updates `package.json` version field
- **cargo**: Updates `Cargo.toml` version field
- **python**: Updates `__version__` in Python files
- **maven**: Updates `<version>` tags in `pom.xml`

Configure package files in your `.version-it` config:

```yaml
package-files:
- path: package.json
  manager: npm
- path: Cargo.toml
  manager: cargo
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

## CI Integration

The `auto-bump` command analyzes git commits since the last version tag and determines the appropriate bump based on configured labels:

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
