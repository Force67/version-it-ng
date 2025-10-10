# Monorepo Versioning with version-it

Ever tried managing versions across a bunch of different projects in one big repository? This example shows how version-it handles that beautifully.

We've got a monorepo with multiple components written in different languages, each needing its own versioning strategy. Plus, we want to track an overall "product version" for the whole thing.

## What's in Here

```
monorepo-example/
├── .version-it                 # Global product version config
├── VERSION                     # Current product version (1.0.0)
├── VERSION.txt                 # Human-readable version file
├── demo.sh                     # Monorepo versioning demo
├── bump-demo.sh                # Bump types explanation demo
└── packages/
    ├── app/                    # Python app (semantic versioning)
    │   ├── .version-it
    │   └── version.py
    ├── cpp-app/                # C++ app (semantic versioning)
    │   ├── .version-it
    │   ├── CMakeLists.txt
    │   └── version.txt
    ├── lib/                    # C library (calendar versioning)
    │   ├── .version-it
    │   └── version.h
    ├── rust-lib/               # Rust library (semantic versioning)
    │   ├── .version-it
    │   ├── Cargo.toml
    │   └── version.txt
    └── service/                # Node.js service (timestamp versioning)
        ├── .version-it
        └── package.json
```

## The Two-Level Versioning Approach

### Product-Level Versioning

At the root, we track the overall product version. This is what customers see - "Hey, we're on version 1.1.0 now!"

```bash
cd examples/monorepo-example
version-it bump --bump minor  # 1.0.0 → 1.1.0
```

This updates both `VERSION` and `VERSION.txt` files.

### Component-Level Versioning

Each component has its own version that can change independently. Maybe your Python app releases more frequently than your C library.

## Understanding Bump Types

Version-it supports different bump types that work differently depending on your versioning scheme:

- **patch**: Small changes, bug fixes
- **minor**: New features, backwards-compatible changes
- **major**: Breaking changes, major rewrites

Run `./bump-demo.sh` to see exactly how these work with each versioning scheme!

```bash
# Python app - semantic versioning
cd packages/app
version-it bump --bump patch  # 1.0.0 → 1.0.1

# C++ app - semantic versioning
cd packages/cpp-app
version-it bump --bump patch  # 1.0.0 → 1.0.1

# C library - calendar versioning
cd packages/lib
version-it bump --bump minor  # 25.10.01 → 25.11.01

# Rust library - semantic versioning
cd packages/rust-lib
version-it bump --bump patch  # 0.1.0 → 0.1.1

# Node.js service - timestamp versioning
cd packages/service
version-it bump --bump patch  # Updates to current timestamp
```

## Component Details

- **Python App**: Uses semantic versioning (1.2.3), generates `version.py`
- **C++ App**: Uses semantic versioning (1.2.3), updates `CMakeLists.txt`
- **C Library**: Calendar versioning (YY.MM.DD), generates `version.h`
- **Rust Library**: Semantic versioning (0.x.x), updates `Cargo.toml`
- **Node.js Service**: Timestamp versioning, updates `package.json`

Each component tracks its version in a `version.txt` file and updates its language-specific package files automatically.

## Try It Out

### Quick Demos

**Monorepo Demo**: See components version independently:
```bash
cd examples/monorepo-example
./demo.sh
```

**Bump Types Demo**: Learn how patch/minor/major bumps work:
```bash
cd examples/monorepo-example
./bump-demo.sh
```

The monorepo demo shows:
- Initial versions across all components
- Global product version bump
- Multiple bumps for each component
- How versions stay independent

### Manual Versioning

Want to do it yourself? Here's how to version everything:

```bash
cd examples/monorepo-example

# Bump the product version first
version-it bump --bump minor

# Then update each component
for dir in packages/*; do
  echo "Versioning $dir..."
  (cd "$dir" && version-it bump --bump patch)
done
```

## Why This Matters

In a real monorepo, you might have:
- A web app that releases weekly
- A mobile SDK that follows calendar versioning
- Backend services with timestamp-based versions
- Libraries that use semantic versioning

Version-it lets each component do its own thing while keeping track of the big picture. No more version conflicts or manual coordination nightmares!