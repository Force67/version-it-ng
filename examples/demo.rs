use version_it_core::{VersionComposer, ComposerConfig, VersionBlock, BlockType, VersionTemplate};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Version Crafting Demo ===\n");

    // Create a simple configuration with various templates
    let mut config = ComposerConfig::new();

    // Add some counters
    config.set_counter("build", 42);
    config.set_counter("release", 5);
    config.set_default_template("semantic-release");

    // Add semantic version template
    config.add_template(
        VersionTemplate::new("semantic-release")
            .with_prefix("v")
            .add_block(
                VersionBlock::new("version", BlockType::Semantic {
                    major: Some(1),
                    minor: Some(2),
                    patch: Some(3)
                })
            )
    );

    // Add calendar version template
    config.add_template(
        VersionTemplate::new("calver-release")
            .with_prefix("release-")
            .with_separator("-")
            .add_block(
                VersionBlock::new("date", BlockType::Calver {
                    year: None,
                    month: None,
                    day: None
                }).with_format("YYYY.MM.DD")
            )
            .add_block(
                VersionBlock::new("build", BlockType::Counter {
                    name: "build".to_string()
                })
            )
    );

    // Add complex enterprise template
    config.add_template(
        VersionTemplate::new("enterprise")
            .with_prefix("myapp-")
            .with_separator("-")
            .with_suffix("-GA")
            .add_block(
                VersionBlock::new("version", BlockType::Semantic {
                    major: Some(3),
                    minor: Some(0),
                    patch: Some(0)
                })
            )
            .add_block(
                VersionBlock::new("timestamp", BlockType::Timestamp)
                    .with_format("YYYYMMDD")
            )
            .add_block(
                VersionBlock::new("release", BlockType::Counter {
                    name: "release".to_string()
                })
            )
            .add_block(
                VersionBlock::new("commit", BlockType::Commit)
            )
    );

    // Create composer from config
    let mut composer = VersionComposer::from_config(&config);

    // Generate versions using different templates
    println!("Available templates:");
    for template_name in composer.list_templates() {
        let default_marker = if composer.default_template.as_ref().map_or(false, |d| d == template_name) {
            " (default)"
        } else {
            ""
        };
        println!("  - {}{}", template_name, default_marker);
    }
    println!();

    println!("Generated versions:");

    println!("1. Default template (semantic-release):");
    let version1 = composer.generate_version(None)?;
    println!("   {}", version1);

    println!("2. Calendar version:");
    let version2 = composer.generate_version(Some("calver-release"))?;
    println!("   {}", version2);

    println!("3. Enterprise version:");
    let version3 = composer.generate_version(Some("enterprise"))?;
    println!("   {}", version3);

    println!("\n=== Counter Operations ===");

    println!("Before increment - build counter: {}",
             composer.counters.get("build").copied().unwrap_or(0));

    let new_build = composer.increment_counter("build");
    println!("After increment - build counter: {}", new_build);

    let version4 = composer.generate_version(Some("calver-release"))?;
    println!("New calver version with incremented build: {}", version4);

    println!("\n=== Template with Multiple Blocks ===");

    // Create a template with multiple different block types
    let multi_template = VersionTemplate::new("multi-block")
        .with_prefix("build-")
        .with_separator("-")
        .add_block(
            VersionBlock::new("semantic", BlockType::Semantic {
                major: Some(2),
                minor: Some(1),
                patch: Some(0)
            })
        )
        .add_block(
            VersionBlock::new("branch", BlockType::Branch)
        )
        .add_block(
            VersionBlock::new("date", BlockType::Date {
                format: "%Y-%m-%d".to_string()
            })
        )
        .add_block(
            VersionBlock::new("counter", BlockType::Counter {
                name: "release".to_string()
            })
        );

    composer.add_template(multi_template);
    let version5 = composer.generate_version(Some("multi-block"))?;
    println!("Multi-block version: {}", version5);

    println!("\n=== Demo Complete ===");
    Ok(())
}