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

    // Create navigator (now takes context_window_tokens instead of max_items_per_module)
    let navigator = Navigator::new(&test_project, 10, 4000);

    match navigator {
        Ok(nav) => {
            // Verify we found modules
            let module_count = nav.module_count();
            println!("Found {} modules in test project", module_count);
            assert!(module_count > 0, "Should find at least one module");

            // Explore the codebase — now produces analysis chunks (one or more per module)
            let mut explorer = nav.explore();
            let chunks = explorer.explore();

            println!("Found {} analysis chunks:", chunks.len());
            for chunk in &chunks {
                println!(
                    "  - module: {}, file: {}, structs: {}, fns: {}, impls: {}",
                    chunk.module_path,
                    chunk.file_path.display(),
                    chunk.structs.len(),
                    chunk.functions.len(),
                    chunk.impl_blocks.len(),
                );
            }

            // We should find analysis chunks with content
            assert!(
                !chunks.is_empty(),
                "Should find at least one analysis chunk"
            );

            // Verify raw source is populated (preserves comments for latent invariant detection)
            for chunk in &chunks {
                assert!(
                    !chunk.raw_source.is_empty(),
                    "Chunk for {} should have raw source",
                    chunk.module_path
                );
            }
        }
        Err(e) => {
            panic!("Failed to create navigator: {}", e);
        }
    }
}
