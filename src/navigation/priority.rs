use crate::parser::SelfParam;

use super::AnalysisChunk;

/// Score and sort analysis chunks so high-value targets are processed first.
pub fn prioritize_chunks(mut chunks: Vec<AnalysisChunk>, priority_prefixes: &[String]) -> Vec<AnalysisChunk> {
    chunks.sort_by(|a, b| {
        let sa = score_chunk(a, priority_prefixes);
        let sb = score_chunk(b, priority_prefixes);
        sb.cmp(&sa) // descending
    });
    chunks
}

fn score_chunk(chunk: &AnalysisChunk, priority_prefixes: &[String]) -> i32 {
    let mut score: i32 = 0;

    // +1000: module path matches a priority prefix
    for prefix in priority_prefixes {
        if chunk.module_path.contains(prefix) {
            score += 1000;
            break;
        }
    }

    // +50: struct with PhantomData (typestate indicator)
    for s in &chunk.structs {
        if s.has_phantom_data {
            score += 50;
        }
    }

    // +40: impl block for Drop trait
    for imp in &chunk.impl_blocks {
        if imp.trait_name.as_deref() == Some("Drop") {
            score += 40;
        }
    }

    // +30: method with SelfParam::Owned (consuming/transition)
    for imp in &chunk.impl_blocks {
        for m in &imp.methods {
            if m.self_param == Some(SelfParam::Owned) {
                score += 30;
                break;
            }
        }
    }

    // +20: unsafe methods
    for imp in &chunk.impl_blocks {
        for m in &imp.methods {
            if m.is_unsafe {
                score += 20;
                break;
            }
        }
    }
    for f in &chunk.functions {
        if f.is_unsafe {
            score += 20;
            break;
        }
    }

    // +10: raw source contains safety/invariant keywords
    let lower = chunk.raw_source.to_lowercase();
    let keywords = ["safety", "must", "invariant", "precondition"];
    for kw in &keywords {
        if lower.contains(kw) {
            score += 10;
            break;
        }
    }

    score
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{StructDef, SourceLocation, Visibility};

    fn make_chunk(module_path: &str) -> AnalysisChunk {
        AnalysisChunk {
            chunk_id: module_path.to_string(),
            module_path: module_path.to_string(),
            file_path: "test.rs".into(),
            raw_source: String::new(),
            structs: vec![],
            enums: vec![],
            functions: vec![],
            traits: vec![],
            impl_blocks: vec![],
            sibling_summary: None,
        }
    }

    #[test]
    fn test_priority_prefix_boost() {
        let mut c1 = make_chunk("std::sync::mutex");
        let mut c2 = make_chunk("std::fmt::display");
        let chunks = prioritize_chunks(vec![c2, c1], &["sync".to_string()]);
        assert_eq!(chunks[0].module_path, "std::sync::mutex");
    }

    #[test]
    fn test_phantom_data_boost() {
        let mut c1 = make_chunk("mod_a");
        c1.structs.push(StructDef {
            name: "Foo".to_string(),
            generics: String::new(),
            fields: vec![],
            visibility: Visibility::Public,
            doc_comment: None,
            has_phantom_data: true,
            source_location: SourceLocation { file_path: "test.rs".to_string(), line: 1 },
        });
        let c2 = make_chunk("mod_b");
        let chunks = prioritize_chunks(vec![c2, c1], &[]);
        assert_eq!(chunks[0].module_path, "mod_a");
    }

    #[test]
    fn test_priority_prefix_1000_boost() {
        let c = make_chunk("std::sync::mutex");
        let score = score_chunk(&c, &["sync".to_string()]);
        assert!(score >= 1000, "Priority prefix should give at least 1000 points, got {}", score);
    }

    #[test]
    fn test_safety_keyword_boost() {
        let mut c1 = make_chunk("mod_a");
        c1.raw_source = "// SAFETY: must hold lock".to_string();
        let c2 = make_chunk("mod_b");
        let chunks = prioritize_chunks(vec![c2, c1], &[]);
        assert_eq!(chunks[0].module_path, "mod_a");
    }
}
