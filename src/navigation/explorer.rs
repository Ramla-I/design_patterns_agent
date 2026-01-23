use std::collections::{HashSet, VecDeque};

use crate::parser::{CodeItem, ImplBlock, ModuleGraph};
use super::context::{CodeContext, InterestingItem};

/// Top-down explorer for navigating the module hierarchy
pub struct Explorer<'a> {
    module_graph: &'a ModuleGraph,
    max_depth: usize,
    max_items_per_module: usize,
    visited_modules: HashSet<String>,
}

impl<'a> Explorer<'a> {
    pub fn new(module_graph: &'a ModuleGraph, max_depth: usize, max_items_per_module: usize) -> Self {
        Self {
            module_graph,
            max_depth,
            max_items_per_module,
            visited_modules: HashSet::new(),
        }
    }

    /// Explore the codebase and yield interesting items
    pub fn explore(&mut self) -> Vec<CodeContext> {
        let mut results = Vec::new();
        let mut queue = VecDeque::new();

        // Start from the root module
        let root = self.module_graph.root();
        queue.push_back((root.name.clone(), 0));

        while let Some((module_name, depth)) = queue.pop_front() {
            if depth > self.max_depth || self.visited_modules.contains(&module_name) {
                continue;
            }

            self.visited_modules.insert(module_name.clone());

            if let Some(module) = self.module_graph.get_module(&module_name) {
                // Extract interesting items from this module
                let interesting = self.find_interesting_items(&module.items);
                for item in interesting.into_iter().take(self.max_items_per_module) {
                    results.push(CodeContext {
                        item,
                        surrounding_code: String::new(), // TODO: Extract actual surrounding code
                        module_path: module_name.clone(),
                    });
                }

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

    fn find_interesting_items(&self, items: &[CodeItem]) -> Vec<InterestingItem> {
        let mut interesting = Vec::new();

        // Collect all impl blocks for reference
        let impl_blocks: Vec<&ImplBlock> = items
            .iter()
            .filter_map(|item| {
                if let CodeItem::Impl(impl_block) = item {
                    Some(impl_block)
                } else {
                    None
                }
            })
            .collect();

        // Check each item for interestingness
        for item in items {
            if let Some(interesting_item) = self.is_interesting(item, &impl_blocks) {
                interesting.push(interesting_item);
            }
        }

        interesting
    }

    fn is_interesting(&self, item: &CodeItem, impl_blocks: &[&ImplBlock]) -> Option<InterestingItem> {
        // Find related impl blocks for structs
        let related_impls: Vec<ImplBlock> = match item {
            CodeItem::Struct(s) => impl_blocks
                .iter()
                .filter(|impl_block| impl_block.type_name.contains(&s.name))
                .map(|&ib| ib.clone())
                .collect(),
            _ => vec![],
        };

        InterestingItem::from_code_item(item, &related_impls)
    }

    pub fn visited_count(&self) -> usize {
        self.visited_modules.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{StructDef, Visibility, SourceLocation};
    use std::fs;
    use tempfile::TempDir;

    fn create_test_crate_with_typestate() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir).unwrap();

        fs::write(
            src_dir.join("lib.rs"),
            r#"
use std::marker::PhantomData;

pub struct State<S> {
    _state: PhantomData<S>,
}
"#,
        )
        .unwrap();

        temp_dir
    }

    #[test]
    fn test_explorer_creation() {
        let temp_dir = create_test_crate_with_typestate();
        let crate_root = crate::parser::find_crate_root(temp_dir.path()).unwrap();
        let module_graph = ModuleGraph::from_crate_root(&crate_root).unwrap();

        let explorer = Explorer::new(&module_graph, 10, 50);
        assert_eq!(explorer.visited_count(), 0);
    }

    #[test]
    fn test_explorer_finds_interesting_items() {
        let temp_dir = create_test_crate_with_typestate();
        let crate_root = crate::parser::find_crate_root(temp_dir.path()).unwrap();
        let module_graph = ModuleGraph::from_crate_root(&crate_root).unwrap();

        let mut explorer = Explorer::new(&module_graph, 10, 50);
        let results = explorer.explore();

        // Should find the State<S> struct with PhantomData
        assert!(!results.is_empty());
        assert_eq!(explorer.visited_count(), 1);
    }

    #[test]
    fn test_explorer_respects_max_depth() {
        let temp_dir = create_test_crate_with_typestate();
        let crate_root = crate::parser::find_crate_root(temp_dir.path()).unwrap();
        let module_graph = ModuleGraph::from_crate_root(&crate_root).unwrap();

        let mut explorer = Explorer::new(&module_graph, 0, 50);
        let results = explorer.explore();

        // With max_depth=0, should still visit root but no children
        assert_eq!(explorer.visited_count(), 1);
    }

    #[test]
    fn test_find_interesting_items() {
        let temp_dir = create_test_crate_with_typestate();
        let crate_root = crate::parser::find_crate_root(temp_dir.path()).unwrap();
        let module_graph = ModuleGraph::from_crate_root(&crate_root).unwrap();

        let explorer = Explorer::new(&module_graph, 10, 50);
        let root = module_graph.root();

        let interesting = explorer.find_interesting_items(&root.items);
        assert!(!interesting.is_empty());
    }
}
