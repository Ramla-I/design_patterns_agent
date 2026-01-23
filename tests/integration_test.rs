use design_patterns_agent::navigation::Navigator;
use std::path::PathBuf;

#[test]
fn test_parse_typestate_example_project() {
    // Get the path to our test project
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let test_project = manifest_dir.join("test_projects/typestate_example");

    // Skip if test project doesn't exist
    if !test_project.exists() {
        eprintln!("Test project not found, skipping test");
        return;
    }

    // Create navigator
    let navigator = Navigator::new(&test_project, 10, 50);

    match navigator {
        Ok(nav) => {
            // Verify we found modules
            let module_count = nav.module_count();
            println!("Found {} modules in test project", module_count);
            assert!(module_count > 0, "Should find at least one module");

            // Explore the codebase
            let mut explorer = nav.explore();
            let interesting_items = explorer.explore();

            println!("Found {} interesting items:", interesting_items.len());
            for item in &interesting_items {
                println!("  - {} ({})", item.module_path, item.item.reason());
            }

            // We should find interesting items:
            // - FileHandle<S> with PhantomData (typestate)
            // - Connection<S> with PhantomData (typestate)
            // - Builder<S> with PhantomData (typestate)
            // - Resource with Drop impl (linear type)
            // - ResourceGuard with Drop impl (linear type)
            assert!(
                interesting_items.len() >= 3,
                "Should find at least 3 interesting items (found {})",
                interesting_items.len()
            );

            // Verify we found some typestate patterns
            let typestate_count = interesting_items
                .iter()
                .filter(|item| {
                    matches!(
                        item.item,
                        design_patterns_agent::navigation::InterestingItem::TypeStateCandidate { .. }
                    )
                })
                .count();

            println!("Found {} typestate candidates", typestate_count);
            assert!(typestate_count > 0, "Should find at least one typestate pattern");

            // Verify we found some linear types
            let linear_type_count = interesting_items
                .iter()
                .filter(|item| {
                    matches!(
                        item.item,
                        design_patterns_agent::navigation::InterestingItem::LinearTypeCandidate { .. }
                    )
                })
                .count();

            println!("Found {} linear type candidates", linear_type_count);
            assert!(linear_type_count > 0, "Should find at least one linear type");
        }
        Err(e) => {
            panic!("Failed to create navigator: {}", e);
        }
    }
}
