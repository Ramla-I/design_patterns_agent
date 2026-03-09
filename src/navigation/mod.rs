mod explorer;
mod context;
pub mod priority;

pub use explorer::Explorer;
pub use context::AnalysisChunk;

use anyhow::Result;
use std::path::Path;

use crate::parser::{ModuleGraph, ParseFailure};

/// Navigate a codebase and extract module-level analysis chunks
pub struct Navigator {
    module_graph: ModuleGraph,
    max_depth: usize,
    context_window_tokens: usize,
}

impl Navigator {
    pub fn new(codebase_path: &Path, max_depth: usize, context_window_tokens: usize) -> Result<Self> {
        let crate_root = crate::parser::find_crate_root(codebase_path)?;
        let module_graph = ModuleGraph::from_crate_root(&crate_root)?;

        Ok(Self {
            module_graph,
            max_depth,
            context_window_tokens,
        })
    }

    /// Create a navigator for multi-crate workspaces (e.g., rust stdlib library/)
    pub fn new_multi_crate(codebase_path: &Path, max_depth: usize, context_window_tokens: usize) -> Result<Self> {
        let crate_roots = crate::parser::find_workspace_crates(codebase_path)?;
        if crate_roots.is_empty() {
            anyhow::bail!("No crates found under {}", codebase_path.display());
        }
        println!("  Found {} crates: {}", crate_roots.len(),
            crate_roots.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>().join(", "));

        let module_graph = ModuleGraph::from_workspace(crate_roots)?;

        Ok(Self {
            module_graph,
            max_depth,
            context_window_tokens,
        })
    }

    pub fn explore(&self) -> Explorer {
        Explorer::new(&self.module_graph, self.max_depth, self.context_window_tokens)
    }

    pub fn module_count(&self) -> usize {
        self.module_graph.modules().count()
    }

    pub fn parse_failures(&self) -> Vec<ParseFailure> {
        self.module_graph.parse_failures.clone()
    }
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
pub struct MainStruct {
    field: i32,
}

impl MainStruct {
    pub fn new(val: i32) -> Self {
        Self { field: val }
    }
}
"#,
        )
        .unwrap();

        temp_dir
    }

    #[test]
    fn test_navigator_creation() {
        let temp_dir = create_test_crate();
        let navigator = Navigator::new(temp_dir.path(), 10, 4000);
        assert!(navigator.is_ok());
    }

    #[test]
    fn test_navigator_module_count() {
        let temp_dir = create_test_crate();
        let navigator = Navigator::new(temp_dir.path(), 10, 4000).unwrap();
        assert_eq!(navigator.module_count(), 1);
    }

    #[test]
    fn test_navigator_explore() {
        let temp_dir = create_test_crate();
        let navigator = Navigator::new(temp_dir.path(), 10, 4000).unwrap();
        let _explorer = navigator.explore();
    }
}
