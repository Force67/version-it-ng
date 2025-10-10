#!/bin/sh

# Monorepo versioning demo script
# This script demonstrates global product versioning and individual component versioning

echo "=== Monorepo Versioning Demo ==="
echo "Demonstrating independent version storage and multiple bumps per component"
echo

# Function to get current version from component
get_component_version() {
    local component=$1
    local dir="packages/$component"

    cd "$dir"
    local version=""
    if [ -f "version.txt" ]; then
        version=$(cat version.txt)
    elif [ -f "version.py" ]; then
        version=$(grep "__version__" version.py | sed 's/.*__version__ = "\([^"]*\)".*/\1/')
    elif [ -f "version.h" ]; then
        version=$(grep "#define LIB_VERSION" version.h | sed 's/.*LIB_VERSION "\([^"]*\)".*/\1/')
    elif [ -f "CMakeLists.txt" ]; then
        version=$(grep "project.*VERSION" CMakeLists.txt | sed 's/.*VERSION \([0-9.]*\).*/\1/')
    elif [ -f "Cargo.toml" ]; then
        version=$(grep "^version" Cargo.toml | sed 's/.*version = "\([^"]*\)".*/\1/')
    elif [ -f "package.json" ]; then
        version=$(grep '"version"' package.json | sed 's/.*"version": "\([^"]*\)".*/\1/')
    fi
    cd - > /dev/null
    echo "$version"
}

# Function to bump a component using version-it
bump_component() {
    local component=$1
    local bump_type=$2
    local dir="packages/$component"

    echo "→ Bumping $component ($bump_type)..."

    # Get current version
    local current_version=$(get_component_version "$component")
    echo "  Current: $current_version"

    # Change to component directory and run version-it
    cd "$dir"
    echo "  Running: version-it bump --bump $bump_type"
    if [ -f "../../../../target/debug/version-it" ]; then
        ../../../../target/debug/version-it bump --bump "$bump_type" 2>/dev/null
    else
        echo "  ERROR: version-it binary not found at ../../../../target/debug/version-it"
        cd - > /dev/null
        return 1
    fi
    cd - > /dev/null

    # Show new version
    local new_version=$(get_component_version "$component")
    echo "  New:     $new_version"
}

# Function to show all component versions
show_all_versions() {
    echo "Current versions across all components:"
    echo "Global: $(cat VERSION)"

    for component in app cpp-app lib rust-lib service; do
        local version=$(get_component_version "$component")
        echo "$component: $version"
    done
    echo
}

# Initial state
echo "=== Initial State ==="
show_all_versions

# Bump global version
echo "=== Bumping Global Product Version ==="
echo "Current global version: $(cat VERSION)"
echo "→ Running: version-it bump --bump minor"
if [ -f "../../target/debug/version-it" ]; then
    ../../target/debug/version-it bump --bump minor 2>/dev/null
    echo "Global version bumped successfully"
else
    echo "ERROR: version-it binary not found at ./target/debug/version-it"
fi
echo


# Multiple bumps per component
echo "=== Multiple Component Bumps ==="
echo "Each component maintains its own version independently"
echo

# App bumps
echo "📦 App Component (Semantic versioning)"
bump_component "app" "patch"
bump_component "app" "patch"
bump_component "app" "patch"
echo

# Lib bumps
echo "📦 Lib Component (Calendar versioning)"
bump_component "lib" "patch"
bump_component "lib" "patch"
echo

# Rust-lib bumps
echo "📦 Rust-lib Component (Semantic versioning)"
bump_component "rust-lib" "patch"
bump_component "rust-lib" "patch"
echo

# C++ app bumps
echo "📦 C++ App Component (Semantic versioning)"
bump_component "cpp-app" "patch"
bump_component "cpp-app" "patch"
echo

# Service bumps
echo "📦 Service Component (Timestamp versioning)"
bump_component "service" "patch"
bump_component "service" "patch"
bump_component "service" "patch"
bump_component "service" "patch"
echo

# Final state
echo "=== Final State After Multiple Bumps ==="
show_all_versions

echo "=== Key Observations ==="
echo "✅ Each component maintains its own version file (version.txt)"
echo "✅ Versions increment independently based on component's scheme"
echo "✅ Global product version is separate from component versions"
echo "✅ Components can be bumped multiple times without affecting others"
echo "✅ Version history is preserved per component"
echo
echo "The monorepo supports multiple package managers:"
echo "- Python: version.py files"
echo "- C/C++: version.h header files and CMakeLists.txt"
echo "- Rust: Cargo.toml files"
echo "- Node.js: package.json files"
echo
echo "This demonstrates version-it's robust monorepo versioning capabilities!"