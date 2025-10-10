#!/bin/sh

# Version bump types demonstration
# Shows how different bump types work with different versioning schemes

echo "=== Version Bump Types Demonstration ==="
echo "Showing how patch/minor/major bumps work with different versioning schemes"
echo

# Function to demonstrate bumps for a scheme
demo_bumps() {
    local scheme=$1
    local initial_version=$2
    local description=$3

    echo "🎯 $scheme Versioning ($description)"
    echo "Starting version: $initial_version"
    echo

    # Simulate different bump types
    case $scheme in
        "Semantic")
            echo "📈 Patch bump (1.2.3 → 1.2.4):"
            echo "   Increments the patch number"
            echo "   Use for: bug fixes, small changes"
            echo

            echo "📈 Minor bump (1.2.3 → 1.3.0):"
            echo "   Increments minor, resets patch to 0"
            echo "   Use for: new features, backwards-compatible changes"
            echo

            echo "📈 Major bump (1.2.3 → 2.0.0):"
            echo "   Increments major, resets minor/patch to 0"
            echo "   Use for: breaking changes, major rewrites"
            echo
            ;;

        "Calendar")
            echo "📅 Patch bump (25.10.15 → 25.10.16):"
            echo "   Increments the day"
            echo "   Use for: daily builds, incremental releases"
            echo

            echo "📅 Minor bump (25.10.15 → 25.11.01):"
            echo "   Increments month, resets day to 01"
            echo "   Use for: monthly releases, feature milestones"
            echo

            echo "📅 Major bump (25.10.15 → 26.01.01):"
            echo "   Increments year, resets month/day to 01"
            echo "   Use for: yearly releases, major version boundaries"
            echo
            ;;

        "Timestamp")
            echo "⏰ Patch bump (20251005220904 → 202510052210XX):"
            echo "   Updates to current timestamp"
            echo "   Use for: nightly builds, CI releases"
            echo "   Note: Minor/Major don't apply - always uses current time"
            echo
            ;;

        "Build")
            echo "🔢 Patch bump (1.2.3.456 → 1.2.3.457):"
            echo "   Increments build number"
            echo "   Use for: continuous integration builds"
            echo

            echo "🔢 Minor bump (1.2.3.456 → 1.2.4.0):"
            echo "   Increments minor, resets build to 0"
            echo "   Use for: feature releases"
            echo

            echo "🔢 Major bump (1.2.3.456 → 1.3.0.0):"
            echo "   Increments major, resets minor/build to 0"
            echo "   Use for: major releases"
            echo
            ;;
    esac

    echo "─────────────────────────────────────"
    echo
}

# Demonstrate each versioning scheme
demo_bumps "Semantic" "1.2.3" "major.minor.patch"
demo_bumps "Calendar" "25.10.15" "YY.MM.DD"
demo_bumps "Timestamp" "20251005220904" "YYYYMMDDHHMMSS"
demo_bumps "Build" "1.2.3.456" "major.minor.patch.build"

echo "=== Real Examples from the Monorepo ==="
echo

# Show actual examples from the monorepo components
echo "🔍 From our monorepo components:"
echo

echo "Python App (Semantic):"
echo "  Current: $(cd packages/app && cat version.txt)"
echo "  Patch bump would → $(cd packages/app && awk -F. '{print $1"."$2"."($3+1)}' version.txt)"
echo "  Minor bump would → $(cd packages/app && awk -F. '{print $1"."($2+1)".0"}' version.txt)"
echo

echo "C++ App (Semantic):"
echo "  Current: $(cd packages/cpp-app && cat version.txt)"
echo "  Patch bump would → $(cd packages/cpp-app && awk -F. '{print $1"."$2"."($3+1)}' version.txt)"
echo

echo "C Library (Calendar):"
echo "  Current: $(cd packages/lib && cat version.txt)"
echo "  Patch bump would → $(cd packages/lib && awk -F. '{print $1"."$2"."($3+1)}' version.txt)"
echo "  Minor bump would → $(cd packages/lib && awk -F. '{print $1"."($2+1)".01"}' version.txt)"
echo

echo "Rust Library (Semantic):"
echo "  Current: $(cd packages/rust-lib && cat version.txt)"
echo "  Patch bump would → $(cd packages/rust-lib && awk -F. '{print $1"."$2"."($3+1)}' version.txt)"
echo

echo "Node.js Service (Timestamp):"
echo "  Current: $(cd packages/service && cat version.txt)"
echo "  Patch bump would → $(date +%Y%m%d%H%M%S) (current timestamp)"
echo

echo "=== Bump Type Guidelines ==="
echo
echo "🎯 PATCH bumps:"
echo "   • Bug fixes"
echo "   • Small improvements"
echo "   • Documentation updates"
echo "   • Internal changes"
echo
echo "🎯 MINOR bumps:"
echo "   • New features"
echo "   • Backwards-compatible changes"
echo "   • API additions"
echo "   • Feature enhancements"
echo
echo "🎯 MAJOR bumps:"
echo "   • Breaking changes"
echo "   • API removals"
echo "   • Major rewrites"
echo "   • Fundamental changes"
echo
echo "💡 Pro tip: Use 'version-it next --bump <type>' to preview changes before applying them!"