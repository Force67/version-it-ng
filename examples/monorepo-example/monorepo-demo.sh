#!/bin/sh

# Monorepo batch processing demo
# Shows how to bump all components at once

echo "=== Monorepo Batch Processing Demo ==="
echo "Using the new 'version-it monorepo' command to process all components at once"
echo

echo "🔍 Current state of all components:"
echo "Global: $(cat VERSION)"
echo "App: $(cd packages/app && cat version.txt)"
echo "C++ App: $(cd packages/cpp-app && cat version.txt)"
echo "Lib: $(cd packages/lib && cat version.txt)"
echo "Rust Lib: $(cd packages/rust-lib && cat version.txt)"
echo "Service: $(cd packages/service && cat version.txt)"
echo

echo "🚀 Running: version-it monorepo --bump patch --dry-run"
echo "This will show what would happen without making changes..."
echo

# Run the monorepo command in dry-run mode
if command -v ../../target/debug/version-it >/dev/null 2>&1 || [ -f "../../target/debug/version-it" ]; then
    ../../target/debug/version-it monorepo --bump patch --dry-run
else
    echo "⚠️  version-it command not found"
    echo "   For this demo, we'll simulate the output..."
    echo
    echo "📦 Processing: packages/app"
    echo "  📋 Current version: 1.0.0"
    echo "  🎯 Next version: 1.0.1"
    echo "  🔍 Would bump to: 1.0.1"
    echo
    echo "📦 Processing: packages/cpp-app"
    echo "  📋 Current version: 1.0.0"
    echo "  🎯 Next version: 1.0.1"
    echo "  🔍 Would bump to: 1.0.1"
    echo
    echo "📦 Processing: packages/lib"
    echo "  📋 Current version: 25.10.01"
    echo "  🎯 Next version: 25.10.2"
    echo "  🔍 Would bump to: 25.10.2"
    echo
    echo "📦 Processing: packages/rust-lib"
    echo "  📋 Current version: 0.1.0"
    echo "  🎯 Next version: 0.1.1"
    echo "  🔍 Would bump to: 0.1.1"
    echo
    echo "📦 Processing: packages/service"
    echo "  📋 Current version: 20251005220904"
    echo "  🎯 Next version: [current timestamp]"
    echo "  🔍 Would bump to: [current timestamp]"
    echo
    echo "📊 Monorepo bump summary:"
    echo "  ✅ packages/app: 1.0.1"
    echo "  ✅ packages/cpp-app: 1.0.1"
    echo "  ✅ packages/lib: 25.10.2"
    echo "  ✅ packages/rust-lib: 0.1.1"
    echo "  ✅ packages/service: [current timestamp]"
    echo
    echo "📈 Results: 5 successful, 0 failed"
fi

echo
echo "💡 Key Benefits of Monorepo Mode:"
echo "  • Single command processes all components"
echo "  • Consistent bump type across the entire monorepo"
echo "  • Atomic commits and tags for the whole release"
echo "  • No need to cd between directories"
echo "  • Centralized configuration in root .version-it"
echo
echo "🔧 Configuration in .version-it:"
echo "  subprojects:"
echo "    - path: packages/app"
echo "    - path: packages/cpp-app"
echo "    - path: packages/lib"
echo "    - path: packages/rust-lib"
echo "    - path: packages/service"
echo
echo "📝 Usage:"
echo "  version-it monorepo --bump patch        # Bump all components"
echo "  version-it monorepo --bump minor --commit # Bump + commit"
echo "  version-it monorepo --bump major --create-tag --commit  # Full release"
echo
echo "  # Or from the built binary:"
echo "  ../../target/debug/version-it monorepo --bump patch --dry-run"