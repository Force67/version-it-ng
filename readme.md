# version-it

A configurable versioning tool for CI pipelines supporting multiple versioning schemes.

## Versioning Schemes

- **semantic**: Standard semantic versioning (1.2.3)
- **calver**: Calendar versioning (25.10.01 for Oct 1, 2025)
- **timestamp**: Timestamp-based (20251005220904)
- **commit**: Git commit hash-based (abc1234)

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

# Automatically bump based on commits
version-it auto-bump
# Analyzes git commits since last version tag and bumps accordingly
```

## Configuration

Create a `.version-it` file in your project:

```yaml
versioning-scheme: calver
first-version: 25.10.01
version-headers:
- language: c
  path: include/version.h
  template: |
    #ifndef VERSION_H
    #define VERSION_H
    #define VERSION "{{version}}"
    #endif
```

## Templates

Templates use Handlebars syntax. Available variables:
- `{{version}}`: The current version string
- `{{scheme}}`: The versioning scheme (semantic, calver, etc.)

Default templates are provided for supported languages, but you can customize with the `template` field.

See `examples/templates/` for sample templates.

## Development

Use Nix for development:

```bash
nix develop
cargo build
cargo test
```

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
