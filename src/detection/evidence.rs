use crate::navigation::AnalysisChunk;

/// Extract code evidence for LLM analysis from an analysis chunk.
/// Prefers raw source (preserves comments) over reconstructed AST.
pub struct EvidenceExtractor;

impl EvidenceExtractor {
    /// Format a chunk's content for LLM consumption.
    /// Uses raw source when available, falls back to structured reconstruction.
    pub fn format_chunk(chunk: &AnalysisChunk) -> String {
        // Prefer raw source — it preserves comments which are critical for latent invariant detection
        if !chunk.raw_source.is_empty() {
            let mut output = String::new();

            if let Some(ref sibling) = chunk.sibling_summary {
                output.push_str(&format!("// Note: {}\n\n", sibling));
            }

            output.push_str(&chunk.raw_source);
            return output;
        }

        // Fallback: reconstruct from structured data
        Self::reconstruct_from_ast(chunk)
    }

    fn reconstruct_from_ast(chunk: &AnalysisChunk) -> String {
        let mut output = String::new();

        if let Some(ref sibling) = chunk.sibling_summary {
            output.push_str(&format!("// Note: {}\n\n", sibling));
        }

        for s in &chunk.structs {
            if let Some(doc) = &s.doc_comment {
                for line in doc.lines() {
                    output.push_str(&format!("///{}\n", line));
                }
            }
            output.push_str(&format!("pub struct {}{} {{\n", s.name, s.generics));
            for field in &s.fields {
                output.push_str(&format!("    {}: {},\n", field.name, field.ty));
            }
            output.push_str("}\n\n");
        }

        for e in &chunk.enums {
            if let Some(doc) = &e.doc_comment {
                for line in doc.lines() {
                    output.push_str(&format!("///{}\n", line));
                }
            }
            output.push_str(&format!("pub enum {}{} {{\n", e.name, e.generics));
            for variant in &e.variants {
                output.push_str(&format!("    {},\n", variant.name));
            }
            output.push_str("}\n\n");
        }

        for t in &chunk.traits {
            if let Some(doc) = &t.doc_comment {
                for line in doc.lines() {
                    output.push_str(&format!("///{}\n", line));
                }
            }
            output.push_str(&format!("pub trait {} {{\n", t.name));
            for method in &t.methods {
                output.push_str(&format!("    fn {}(...);\n", method));
            }
            output.push_str("}\n\n");
        }

        for imp in &chunk.impl_blocks {
            if let Some(trait_name) = &imp.trait_name {
                output.push_str(&format!("impl {} for {} {{\n", trait_name, imp.type_name));
            } else {
                output.push_str(&format!("impl {} {{\n", imp.type_name));
            }
            for method in &imp.methods {
                if let Some(doc) = &method.doc_comment {
                    for line in doc.lines() {
                        output.push_str(&format!("    ///{}\n", line));
                    }
                }
                if let Some(body) = &method.body_summary {
                    output.push_str(&format!("    {} {{\n        {}\n    }}\n", method.signature, body));
                } else {
                    output.push_str(&format!("    {} {{ ... }}\n", method.signature));
                }
            }
            output.push_str("}\n\n");
        }

        for f in &chunk.functions {
            if let Some(doc) = &f.doc_comment {
                for line in doc.lines() {
                    output.push_str(&format!("///{}\n", line));
                }
            }
            if let Some(body) = &f.body_summary {
                output.push_str(&format!("{} {{\n    {}\n}}\n\n", f.signature, body));
            } else {
                output.push_str(&format!("{} {{ ... }}\n\n", f.signature));
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{StructDef, Field, SourceLocation, Visibility};
    use std::path::PathBuf;

    #[test]
    fn test_format_chunk_prefers_raw_source() {
        let chunk = AnalysisChunk {
            module_path: "test".to_string(),
            file_path: PathBuf::from("test.rs"),
            raw_source: "// Must call init before use\nfn init() {}\nfn use_it() {}".to_string(),
            structs: vec![],
            enums: vec![],
            functions: vec![],
            traits: vec![],
            impl_blocks: vec![],
            sibling_summary: None,
        };

        let formatted = EvidenceExtractor::format_chunk(&chunk);
        assert!(formatted.contains("Must call init before use"));
    }

    #[test]
    fn test_format_chunk_with_sibling_summary() {
        let chunk = AnalysisChunk {
            module_path: "test".to_string(),
            file_path: PathBuf::from("test.rs"),
            raw_source: "fn foo() {}".to_string(),
            structs: vec![],
            enums: vec![],
            functions: vec![],
            traits: vec![],
            impl_blocks: vec![],
            sibling_summary: Some("Other parts contain: struct Bar, impl Bar (3 methods)".to_string()),
        };

        let formatted = EvidenceExtractor::format_chunk(&chunk);
        assert!(formatted.contains("Note:"));
        assert!(formatted.contains("struct Bar"));
    }

    #[test]
    fn test_reconstruct_from_ast_fallback() {
        let chunk = AnalysisChunk {
            module_path: "test".to_string(),
            file_path: PathBuf::from("test.rs"),
            raw_source: String::new(),
            structs: vec![StructDef {
                name: "Conn".to_string(),
                generics: String::new(),
                fields: vec![Field { name: "open".to_string(), ty: "bool".to_string() }],
                visibility: Visibility::Public,
                doc_comment: Some(" A connection".to_string()),
                has_phantom_data: false,
                source_location: SourceLocation { file_path: "test.rs".to_string(), line: 1 },
            }],
            enums: vec![],
            functions: vec![],
            traits: vec![],
            impl_blocks: vec![],
            sibling_summary: None,
        };

        let formatted = EvidenceExtractor::format_chunk(&chunk);
        assert!(formatted.contains("pub struct Conn"));
        assert!(formatted.contains("open: bool"));
        assert!(formatted.contains("A connection"));
    }
}
