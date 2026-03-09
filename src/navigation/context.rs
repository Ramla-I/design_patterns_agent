use std::path::PathBuf;

use crate::parser::{CodeItem, EnumDef, FunctionDef, ImplBlock, Module, StructDef, TraitDef};

/// A chunk of a module to be analyzed by the LLM.
/// May represent an entire module (if it fits in context) or a cluster of related items.
#[derive(Debug, Clone)]
pub struct AnalysisChunk {
    /// Module path (e.g., "crate::connection")
    pub module_path: String,
    /// File path on disk
    pub file_path: PathBuf,
    /// Raw source code of the relevant section (preserves comments)
    pub raw_source: String,
    /// Structured items in this chunk (for evidence extraction)
    pub structs: Vec<StructDef>,
    pub enums: Vec<EnumDef>,
    pub functions: Vec<FunctionDef>,
    pub traits: Vec<TraitDef>,
    pub impl_blocks: Vec<ImplBlock>,
    /// Brief description of what other chunks from this module contain
    /// (gives the LLM awareness of the full module even when split)
    pub sibling_summary: Option<String>,
}

/// A cluster of related items within a module (used for splitting large modules)
#[derive(Debug, Clone)]
pub struct ItemCluster {
    /// The primary type this cluster is organized around (if any)
    pub anchor_type: Option<String>,
    pub structs: Vec<StructDef>,
    pub enums: Vec<EnumDef>,
    pub functions: Vec<FunctionDef>,
    pub traits: Vec<TraitDef>,
    pub impl_blocks: Vec<ImplBlock>,
}

impl ItemCluster {
    fn estimated_tokens(&self) -> usize {
        let mut chars = 0;
        for s in &self.structs {
            chars += s.name.len() + s.fields.iter().map(|f| f.name.len() + f.ty.len() + 4).sum::<usize>() + 20;
            if let Some(doc) = &s.doc_comment { chars += doc.len(); }
        }
        for e in &self.enums {
            chars += e.name.len() + e.variants.iter().map(|v| v.name.len() + 10).sum::<usize>() + 20;
            if let Some(doc) = &e.doc_comment { chars += doc.len(); }
        }
        for f in &self.functions {
            chars += f.signature.len() + 20;
            if let Some(doc) = &f.doc_comment { chars += doc.len(); }
            if let Some(body) = &f.body_summary { chars += body.len(); }
        }
        for t in &self.traits {
            chars += t.name.len() + t.methods.iter().map(|m| m.len() + 10).sum::<usize>() + 20;
        }
        for imp in &self.impl_blocks {
            chars += imp.type_name.len() + 20;
            for m in &imp.methods {
                chars += m.signature.len() + 20;
                if let Some(body) = &m.body_summary { chars += body.len(); }
            }
        }
        // Rough estimate: 4 chars per token
        chars / 4
    }

    fn type_names_referenced(&self) -> Vec<String> {
        let mut names = Vec::new();
        if let Some(ref anchor) = self.anchor_type {
            names.push(anchor.clone());
        }
        for s in &self.structs { names.push(s.name.clone()); }
        for e in &self.enums { names.push(e.name.clone()); }
        names
    }

    fn summary(&self) -> String {
        let mut parts = Vec::new();
        for s in &self.structs { parts.push(format!("struct {}", s.name)); }
        for e in &self.enums { parts.push(format!("enum {}", e.name)); }
        for t in &self.traits { parts.push(format!("trait {}", t.name)); }
        let fn_count = self.functions.len();
        if fn_count > 0 {
            parts.push(format!("{} free function(s)", fn_count));
        }
        for imp in &self.impl_blocks {
            let method_count = imp.methods.len();
            if let Some(ref tn) = imp.trait_name {
                parts.push(format!("impl {} for {} ({} methods)", tn, imp.type_name, method_count));
            } else {
                parts.push(format!("impl {} ({} methods)", imp.type_name, method_count));
            }
        }
        parts.join(", ")
    }
}

/// Build analysis chunks from a module's items and raw source.
/// Splits into multiple chunks if the module exceeds the token limit.
pub fn build_chunks(
    module: &Module,
    raw_source: &str,
    max_tokens: usize,
) -> Vec<AnalysisChunk> {
    let items = &module.items;

    // Classify items into structured data
    let mut structs = Vec::new();
    let mut enums = Vec::new();
    let mut functions = Vec::new();
    let mut traits = Vec::new();
    let mut impl_blocks = Vec::new();

    for item in items {
        match item {
            CodeItem::Struct(s) => structs.push(s.clone()),
            CodeItem::Enum(e) => enums.push(e.clone()),
            CodeItem::Function(f) => functions.push(f.clone()),
            CodeItem::Trait(t) => traits.push(t.clone()),
            CodeItem::Impl(i) => impl_blocks.push(i.clone()),
            CodeItem::TypeAlias(_) => {} // skip type aliases
        }
    }

    // Skip modules with nothing interesting
    let has_content = !structs.is_empty()
        || !enums.is_empty()
        || !functions.is_empty()
        || !traits.is_empty()
        || impl_blocks.iter().any(|i| !i.methods.is_empty());

    if !has_content {
        return vec![];
    }

    // Build a single cluster first and check if it fits
    let single_cluster = ItemCluster {
        anchor_type: None,
        structs: structs.clone(),
        enums: enums.clone(),
        functions: functions.clone(),
        traits: traits.clone(),
        impl_blocks: impl_blocks.clone(),
    };

    let estimated = single_cluster.estimated_tokens();

    if estimated <= max_tokens {
        // Whole module fits in one chunk
        return vec![AnalysisChunk {
            module_path: module.name.clone(),
            file_path: module.path.clone(),
            raw_source: truncate_source(raw_source, max_tokens * 4),
            structs,
            enums,
            functions,
            traits,
            impl_blocks,
            sibling_summary: None,
        }];
    }

    // Need to split — cluster by type affinity
    let clusters = cluster_by_type_affinity(structs, enums, functions, traits, impl_blocks);

    // If still too large, truncate function bodies within clusters
    let clusters: Vec<ItemCluster> = clusters
        .into_iter()
        .map(|mut c| {
            if c.estimated_tokens() > max_tokens {
                strip_bodies(&mut c);
            }
            c
        })
        .collect();

    // Build summaries for sibling awareness
    let summaries: Vec<String> = clusters.iter().map(|c| c.summary()).collect();

    clusters
        .into_iter()
        .enumerate()
        .map(|(i, cluster)| {
            let sibling_summary = if summaries.len() > 1 {
                let others: Vec<&str> = summaries.iter()
                    .enumerate()
                    .filter(|(j, _)| *j != i)
                    .map(|(_, s)| s.as_str())
                    .collect();
                Some(format!("Other parts of this module contain: {}", others.join("; ")))
            } else {
                None
            };

            // Extract raw source relevant to this cluster's types
            let chunk_source = extract_relevant_source(raw_source, &cluster);

            AnalysisChunk {
                module_path: module.name.clone(),
                file_path: module.path.clone(),
                raw_source: truncate_source(&chunk_source, max_tokens * 4),
                structs: cluster.structs,
                enums: cluster.enums,
                functions: cluster.functions,
                traits: cluster.traits,
                impl_blocks: cluster.impl_blocks,
                sibling_summary,
            }
        })
        .collect()
}

/// Cluster items by type affinity: each struct/enum becomes an anchor,
/// and impl blocks + functions that reference it are grouped together.
fn cluster_by_type_affinity(
    structs: Vec<StructDef>,
    enums: Vec<EnumDef>,
    functions: Vec<FunctionDef>,
    traits: Vec<TraitDef>,
    impl_blocks: Vec<ImplBlock>,
) -> Vec<ItemCluster> {
    let mut clusters: Vec<ItemCluster> = Vec::new();
    let mut used_impl_indices = Vec::new();
    let mut used_fn_indices = Vec::new();

    // Create a cluster for each struct
    for s in &structs {
        let mut cluster = ItemCluster {
            anchor_type: Some(s.name.clone()),
            structs: vec![s.clone()],
            enums: vec![],
            functions: vec![],
            traits: vec![],
            impl_blocks: vec![],
        };

        // Attach impl blocks that reference this struct
        for (idx, imp) in impl_blocks.iter().enumerate() {
            if imp.type_name.contains(&s.name) && !used_impl_indices.contains(&idx) {
                cluster.impl_blocks.push(imp.clone());
                used_impl_indices.push(idx);
            }
        }

        // Attach free functions that reference this struct in their signature
        for (idx, f) in functions.iter().enumerate() {
            if f.signature.contains(&s.name) && !used_fn_indices.contains(&idx) {
                cluster.functions.push(f.clone());
                used_fn_indices.push(idx);
            }
        }

        clusters.push(cluster);
    }

    // Create a cluster for each enum
    for e in &enums {
        let mut cluster = ItemCluster {
            anchor_type: Some(e.name.clone()),
            structs: vec![],
            enums: vec![e.clone()],
            functions: vec![],
            traits: vec![],
            impl_blocks: vec![],
        };

        for (idx, imp) in impl_blocks.iter().enumerate() {
            if imp.type_name.contains(&e.name) && !used_impl_indices.contains(&idx) {
                cluster.impl_blocks.push(imp.clone());
                used_impl_indices.push(idx);
            }
        }

        for (idx, f) in functions.iter().enumerate() {
            if f.signature.contains(&e.name) && !used_fn_indices.contains(&idx) {
                cluster.functions.push(f.clone());
                used_fn_indices.push(idx);
            }
        }

        clusters.push(cluster);
    }

    // Remaining items go into a "loose" cluster
    let remaining_impls: Vec<ImplBlock> = impl_blocks
        .iter()
        .enumerate()
        .filter(|(idx, _)| !used_impl_indices.contains(idx))
        .map(|(_, i)| i.clone())
        .collect();

    let remaining_fns: Vec<FunctionDef> = functions
        .iter()
        .enumerate()
        .filter(|(idx, _)| !used_fn_indices.contains(idx))
        .map(|(_, f)| f.clone())
        .collect();

    if !remaining_impls.is_empty() || !remaining_fns.is_empty() || !traits.is_empty() {
        clusters.push(ItemCluster {
            anchor_type: None,
            structs: vec![],
            enums: vec![],
            functions: remaining_fns,
            traits,
            impl_blocks: remaining_impls,
        });
    }

    // Filter out empty clusters
    clusters.retain(|c| {
        !c.structs.is_empty()
            || !c.enums.is_empty()
            || !c.functions.is_empty()
            || !c.traits.is_empty()
            || c.impl_blocks.iter().any(|i| !i.methods.is_empty())
    });

    clusters
}

/// Strip function bodies from a cluster to reduce token count
fn strip_bodies(cluster: &mut ItemCluster) {
    for f in &mut cluster.functions {
        f.body_summary = None;
    }
    for imp in &mut cluster.impl_blocks {
        for m in &mut imp.methods {
            m.body_summary = None;
        }
    }
}

/// Extract lines from raw source that are relevant to a cluster's types
fn extract_relevant_source(raw_source: &str, cluster: &ItemCluster) -> String {
    let type_names = cluster.type_names_referenced();
    if type_names.is_empty() {
        // For loose clusters, return the whole source (will be truncated later)
        return raw_source.to_string();
    }

    // Find lines mentioning any of the type names and include surrounding context
    let lines: Vec<&str> = raw_source.lines().collect();
    let mut included = vec![false; lines.len()];

    for (i, line) in lines.iter().enumerate() {
        for name in &type_names {
            if line.contains(name.as_str()) {
                // Include 5 lines before and after
                let start = i.saturating_sub(5);
                let end = (i + 6).min(lines.len());
                for j in start..end {
                    included[j] = true;
                }
            }
        }
    }

    let mut result = String::new();
    let mut in_section = false;
    for (i, line) in lines.iter().enumerate() {
        if included[i] {
            if !in_section && !result.is_empty() {
                result.push_str("\n// ... (other code) ...\n\n");
            }
            in_section = true;
            result.push_str(line);
            result.push('\n');
        } else {
            in_section = false;
        }
    }

    result
}

/// Truncate source to approximately fit within a character limit
fn truncate_source(source: &str, max_chars: usize) -> String {
    if source.len() <= max_chars {
        source.to_string()
    } else {
        let mut truncated = source[..max_chars].to_string();
        truncated.push_str("\n// ... (truncated) ...");
        truncated
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{self, SourceLocation, Visibility, Field};

    fn make_struct(name: &str, fields: Vec<(&str, &str)>) -> StructDef {
        StructDef {
            name: name.to_string(),
            generics: String::new(),
            fields: fields.into_iter().map(|(n, t)| Field { name: n.to_string(), ty: t.to_string() }).collect(),
            visibility: Visibility::Public,
            doc_comment: None,
            has_phantom_data: false,
            source_location: SourceLocation { file_path: "test.rs".to_string(), line: 1 },
        }
    }

    fn make_function(name: &str, sig: &str) -> FunctionDef {
        FunctionDef {
            name: name.to_string(),
            signature: sig.to_string(),
            is_method: false,
            self_param: None,
            visibility: Visibility::Public,
            doc_comment: None,
            body_summary: None,
            attributes: vec![],
            is_unsafe: false,
            source_location: SourceLocation { file_path: "test.rs".to_string(), line: 1 },
        }
    }

    fn make_module(name: &str, items: Vec<CodeItem>) -> Module {
        Module {
            name: name.to_string(),
            path: "test.rs".into(),
            items,
            submodules: vec![],
            is_root: false,
        }
    }

    #[test]
    fn test_small_module_single_chunk() {
        let module = make_module("test", vec![
            CodeItem::Struct(make_struct("Conn", vec![("fd", "i32"), ("open", "bool")])),
            CodeItem::Function(make_function("connect", "fn connect() -> Conn")),
        ]);

        let chunks = build_chunks(&module, "// source", 4000);
        assert_eq!(chunks.len(), 1);
        assert!(chunks[0].sibling_summary.is_none());
    }

    #[test]
    fn test_empty_module_no_chunks() {
        let module = make_module("empty", vec![]);
        let chunks = build_chunks(&module, "", 4000);
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_type_affinity_clustering() {
        let imp = ImplBlock {
            trait_name: None,
            type_name: "Conn".to_string(),
            methods: vec![make_function("close", "fn close(&mut self)")],
            source_location: SourceLocation { file_path: "test.rs".to_string(), line: 1 },
        };

        let clusters = cluster_by_type_affinity(
            vec![make_struct("Conn", vec![("fd", "i32")])],
            vec![],
            vec![make_function("unrelated", "fn unrelated()")],
            vec![],
            vec![imp],
        );

        // Should have 2 clusters: Conn+impl, and loose functions
        assert_eq!(clusters.len(), 2);
        assert_eq!(clusters[0].anchor_type.as_deref(), Some("Conn"));
        assert_eq!(clusters[0].impl_blocks.len(), 1);
        assert_eq!(clusters[1].functions.len(), 1);
    }
}
