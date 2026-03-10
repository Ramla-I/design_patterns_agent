use std::collections::HashSet;
use std::collections::VecDeque;

use crate::parser::ModuleGraph;
use super::context::{AnalysisChunk, build_chunks};

/// Top-down explorer that builds module-level analysis chunks
pub struct Explorer<'a> {
    module_graph: &'a ModuleGraph,
    max_depth: usize,
    context_window_tokens: usize,
    visited_modules: HashSet<String>,
}

impl<'a> Explorer<'a> {
    pub fn new(module_graph: &'a ModuleGraph, max_depth: usize, context_window_tokens: usize) -> Self {
        Self {
            module_graph,
            max_depth,
            context_window_tokens,
            visited_modules: HashSet::new(),
        }
    }

    /// Explore the codebase and produce analysis chunks (one or more per module)
    pub fn explore(&mut self) -> Vec<AnalysisChunk> {
        let mut results = Vec::new();
        let mut queue = VecDeque::new();

        let root = self.module_graph.root();
        queue.push_back((root.name.clone(), 0));

        while let Some((module_name, depth)) = queue.pop_front() {
            if depth > self.max_depth || self.visited_modules.contains(&module_name) {
                continue;
            }

            self.visited_modules.insert(module_name.clone());

            if let Some(module) = self.module_graph.get_module(&module_name) {
                // Read raw source for this module (preserves comments)
                let raw_source = std::fs::read_to_string(&module.path).unwrap_or_default();

                // Build analysis chunks for this module
                let chunks = build_chunks(module, &raw_source, self.context_window_tokens);
                results.extend(chunks);

                // Add children to the queue
                if depth < self.max_depth {
                    for child in self.module_graph.children(&module_name) {
                        queue.push_back((child.name.clone(), depth + 1));
                    }
                }
            }
        }

        results
    }

    #[cfg(test)]
    pub fn visited_count(&self) -> usize {
        self.visited_modules.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ModuleGraph;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_crate() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir).unwrap();

        fs::write(
            src_dir.join("lib.rs"),
            r#"
use std::marker::PhantomData;

pub struct Connection {
    fd: i32,
    is_open: bool,
}

impl Connection {
    // Must call connect() before send()
    pub fn connect(&mut self) -> bool {
        if self.is_open { return false; }
        self.is_open = true;
        true
    }

    pub fn send(&self, data: &[u8]) -> Result<(), String> {
        if !self.is_open {
            return Err("not connected".to_string());
        }
        Ok(())
    }

    pub fn close(&mut self) {
        self.is_open = false;
    }
}
"#,
        )
        .unwrap();

        temp_dir
    }

    #[test]
    fn test_explorer_creation() {
        let temp_dir = create_test_crate();
        let crate_root = crate::parser::find_crate_root(temp_dir.path()).unwrap();
        let module_graph = ModuleGraph::from_crate_root(&crate_root).unwrap();

        let explorer = Explorer::new(&module_graph, 10, 4000);
        assert_eq!(explorer.visited_count(), 0);
    }

    #[test]
    fn test_explorer_finds_chunks() {
        let temp_dir = create_test_crate();
        let crate_root = crate::parser::find_crate_root(temp_dir.path()).unwrap();
        let module_graph = ModuleGraph::from_crate_root(&crate_root).unwrap();

        let mut explorer = Explorer::new(&module_graph, 10, 4000);
        let chunks = explorer.explore();

        assert!(!chunks.is_empty());
        assert_eq!(explorer.visited_count(), 1);
        // Raw source should contain comments
        assert!(chunks[0].raw_source.contains("Must call connect()"));
    }

    #[test]
    fn test_explorer_respects_max_depth() {
        let temp_dir = create_test_crate();
        let crate_root = crate::parser::find_crate_root(temp_dir.path()).unwrap();
        let module_graph = ModuleGraph::from_crate_root(&crate_root).unwrap();

        let mut explorer = Explorer::new(&module_graph, 0, 4000);
        let _results = explorer.explore();

        assert_eq!(explorer.visited_count(), 1);
    }
}
