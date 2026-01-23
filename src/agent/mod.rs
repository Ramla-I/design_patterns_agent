use anyhow::Result;
use std::path::Path;

use crate::cli::Config;
use crate::detection::InvariantDetector;
use crate::llm;
use crate::navigation::Navigator;
use crate::report::Report;

/// Main analysis function that orchestrates the invariant discovery process
pub async fn analyze_codebase(path: &Path, config: &Config) -> Result<Report> {
    println!("🔍 Initializing analysis of codebase at: {}", path.display());

    // Create LLM client
    println!("🤖 Connecting to LLM provider: {}", config.llm.provider);
    let llm_client = llm::create_client(
        config.llm.api_key.clone(),
        config.llm.model.clone(),
    )?;

    // Create navigator
    println!("📂 Building module graph...");
    let navigator = Navigator::new(
        path,
        config.exploration.max_depth,
        config.exploration.max_items_per_module,
    )?;

    let module_count = navigator.module_count();
    println!("   Found {} modules", module_count);

    // Explore codebase
    println!("🧭 Exploring codebase...");
    let mut explorer = navigator.explore();
    let interesting_items = explorer.explore();

    println!("   Found {} interesting code items", interesting_items.len());

    // Create detector
    let detector = InvariantDetector::new();

    // Analyze each interesting item
    println!("🔬 Analyzing items for invariants...");
    let mut report = Report::new();
    report.summary.modules_analyzed = module_count;

    let mut next_id = 1;
    for (idx, context) in interesting_items.iter().enumerate() {
        println!("   [{}/{}] Analyzing: {}", idx + 1, interesting_items.len(), context.module_path);

        match detector.detect(context, llm_client.as_ref(), &mut next_id).await {
            Ok(invariants) => {
                for invariant in invariants {
                    println!("      ✓ Found: {}", invariant.title);
                    report.add_invariant(invariant);
                }
            }
            Err(e) => {
                eprintln!("      ✗ Error analyzing item: {}", e);
            }
        }
    }

    println!("\n📊 Analysis complete!");
    println!("   Total invariants discovered: {}", report.summary.total_invariants);
    println!("   - State machine: {}", report.summary.state_machine_count);
    println!("   - Linear type: {}", report.summary.linear_type_count);
    println!("   - Ownership: {}", report.summary.ownership_count);

    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_crate() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir).unwrap();

        fs::write(
            src_dir.join("lib.rs"),
            r#"
pub struct SimpleStruct {
    field: i32,
}
"#,
        )
        .unwrap();

        temp_dir
    }

    #[tokio::test]
    async fn test_analyze_codebase_structure() {
        // This test just verifies the structure compiles
        // Actual LLM testing would require mocking or real API keys
        let temp_dir = create_test_crate();

        // We can't actually run this without a real API key
        // but we can verify the code compiles
        assert!(temp_dir.path().exists());
    }
}
