pub mod octocode;
pub mod queries;

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::navigation::AnalysisChunk;
use octocode::{OctocodeClient, SearchResult};
use queries::INVARIANT_QUERIES;

/// Configuration for semantic search mode
#[derive(Debug, Clone)]
pub struct SearchConfig {
    pub similarity_threshold: f32,
    pub max_results_per_query: usize,
    pub context_window_tokens: usize,
    /// Lines of context to include above and below each match
    pub context_lines: usize,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            similarity_threshold: 0.1,
            max_results_per_query: 20,
            context_window_tokens: 4000,
            context_lines: 30,
        }
    }
}

/// Run semantic search over a codebase and produce analysis chunks
/// for the detection pipeline.
pub async fn search_for_invariants(
    repo_path: &Path,
    config: &SearchConfig,
) -> Result<Vec<AnalysisChunk>> {
    let mut client = OctocodeClient::new(repo_path)
        .await
        .context("Failed to start octocode")?;

    let mut all_results: Vec<SearchResult> = Vec::new();

    for query_def in INVARIANT_QUERIES {
        println!(
            "  Searching: \"{}\" ({})",
            query_def.query, query_def.description
        );

        match client
            .search(
                query_def.query,
                query_def.mode,
                config.similarity_threshold,
                config.max_results_per_query,
            )
            .await
        {
            Ok(results) => {
                println!("    {} matches", results.len());
                all_results.extend(results);
            }
            Err(e) => {
                eprintln!("    Search failed: {}", e);
            }
        }
    }

    client.close().await?;

    println!(
        "  Total raw matches: {} (deduplicating...)",
        all_results.len()
    );

    // Deduplicate and merge overlapping results, then build chunks
    let chunks = results_to_chunks(repo_path, all_results, config)?;

    println!("  Produced {} analysis chunks", chunks.len());

    Ok(chunks)
}

/// A file region identified by search results (possibly merged from multiple hits)
#[derive(Debug, Clone)]
struct FileRegion {
    file_path: PathBuf,
    /// Relative path as returned by octocode (e.g., "src/foo.rs")
    relative_path: String,
    line_start: usize,
    line_end: usize,
    max_similarity: f32,
}

/// Convert raw search results into deduplicated AnalysisChunks.
///
/// Strategy:
/// 1. Group results by file
/// 2. Merge overlapping/adjacent line ranges (with context padding)
/// 3. Read raw source for each merged region
/// 4. Build AnalysisChunks with the raw source
fn results_to_chunks(
    repo_path: &Path,
    results: Vec<SearchResult>,
    config: &SearchConfig,
) -> Result<Vec<AnalysisChunk>> {
    // Group by file
    let mut by_file: HashMap<String, Vec<SearchResult>> = HashMap::new();
    for result in results {
        by_file
            .entry(result.file_path.clone())
            .or_default()
            .push(result);
    }

    let mut chunks = Vec::new();

    for (relative_path, mut file_results) in by_file {
        let file_path = repo_path.join(&relative_path);
        if !file_path.exists() {
            continue;
        }

        // Sort by line_start
        file_results.sort_by_key(|r| r.line_start);

        // Merge overlapping/adjacent regions (with context padding)
        let regions = merge_regions(&file_path, &relative_path, &file_results, config.context_lines);

        // Read the file once
        let full_source = std::fs::read_to_string(&file_path).unwrap_or_default();
        let lines: Vec<&str> = full_source.lines().collect();

        for region in regions {
            // Extract the relevant source window
            let start = region.line_start.saturating_sub(1); // 0-indexed
            let end = region.line_end.min(lines.len());
            let raw_source = lines[start..end].join("\n");

            // Estimate tokens; skip if tiny
            if raw_source.len() < 20 {
                continue;
            }

            // Truncate if exceeds token budget
            let max_chars = config.context_window_tokens * 4;
            let raw_source = if raw_source.len() > max_chars {
                format!("{}...\n// (truncated)", &raw_source[..max_chars])
            } else {
                raw_source
            };

            // Derive module path from relative file path
            let module_path = relative_path
                .trim_end_matches(".rs")
                .replace('/', "::")
                .replace("::mod", "")
                .replace("::lib", "crate")
                .replace("::main", "crate");

            chunks.push(AnalysisChunk {
                module_path,
                file_path: file_path.clone(),
                raw_source,
                // Structured fields left empty — raw source is primary for semantic search mode
                structs: vec![],
                enums: vec![],
                functions: vec![],
                traits: vec![],
                impl_blocks: vec![],
                sibling_summary: None,
            });
        }
    }

    // Sort by file path for deterministic output
    chunks.sort_by(|a, b| a.file_path.cmp(&b.file_path));

    Ok(chunks)
}

/// Merge overlapping or adjacent line ranges within a file.
/// Adds `context_lines` padding above and below each match before merging.
fn merge_regions(
    file_path: &Path,
    relative_path: &str,
    results: &[SearchResult],
    context_lines: usize,
) -> Vec<FileRegion> {
    if results.is_empty() {
        return vec![];
    }

    let mut regions: Vec<FileRegion> = Vec::new();

    for result in results {
        let padded_start = result.line_start.saturating_sub(context_lines);
        let padded_end = result.line_end + context_lines;

        if let Some(last) = regions.last_mut() {
            // Merge if overlapping or adjacent
            if padded_start <= last.line_end + 1 {
                last.line_end = last.line_end.max(padded_end);
                last.max_similarity = last.max_similarity.max(result.similarity);
                continue;
            }
        }

        regions.push(FileRegion {
            file_path: file_path.to_path_buf(),
            relative_path: relative_path.to_string(),
            line_start: padded_start.max(1),
            line_end: padded_end,
            max_similarity: result.similarity,
        });
    }

    regions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_overlapping_regions() {
        let results = vec![
            SearchResult {
                file_path: "src/foo.rs".into(),
                similarity: 0.8,
                line_start: 10,
                line_end: 15,
                snippet: String::new(),
            },
            SearchResult {
                file_path: "src/foo.rs".into(),
                similarity: 0.7,
                line_start: 12,
                line_end: 20,
                snippet: String::new(),
            },
            SearchResult {
                file_path: "src/foo.rs".into(),
                similarity: 0.6,
                line_start: 50,
                line_end: 55,
                snippet: String::new(),
            },
        ];

        let regions = merge_regions(Path::new("src/foo.rs"), "src/foo.rs", &results, 5);

        // First two overlap (10-15 and 12-20 with 5-line padding), third is separate
        assert_eq!(regions.len(), 2);
        assert_eq!(regions[0].line_start, 5); // 10 - 5
        assert_eq!(regions[0].line_end, 25); // 20 + 5
        assert!((regions[0].max_similarity - 0.8).abs() < 0.01);
        assert_eq!(regions[1].line_start, 45); // 50 - 5
        assert_eq!(regions[1].line_end, 60); // 55 + 5
    }

    #[test]
    fn test_merge_adjacent_regions() {
        let results = vec![
            SearchResult {
                file_path: "src/foo.rs".into(),
                similarity: 0.8,
                line_start: 10,
                line_end: 15,
                snippet: String::new(),
            },
            SearchResult {
                file_path: "src/foo.rs".into(),
                similarity: 0.7,
                line_start: 20, // within context_lines=5 of previous end (15+5=20)
                line_end: 25,
                snippet: String::new(),
            },
        ];

        let regions = merge_regions(Path::new("src/foo.rs"), "src/foo.rs", &results, 5);

        // These should merge because padded ranges overlap: [5,20] and [15,30]
        assert_eq!(regions.len(), 1);
    }

    #[test]
    fn test_empty_results() {
        let regions = merge_regions(Path::new("src/foo.rs"), "src/foo.rs", &[], 5);
        assert!(regions.is_empty());
    }
}
