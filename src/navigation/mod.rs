mod explorer;
mod context;

pub use explorer::Explorer;
pub use context::{CodeContext, InterestingItem};

use anyhow::Result;
use std::path::Path;

use crate::parser::{CodeItem, ModuleGraph};

/// Navigate a codebase and extract items for analysis
pub struct Navigator {
    module_graph: ModuleGraph,
    max_depth: usize,
    max_items_per_module: usize,
}

impl Navigator {
    pub fn new(codebase_path: &Path, max_depth: usize, max_items_per_module: usize) -> Result<Self> {
        let crate_root = crate::parser::find_crate_root(codebase_path)?;
        let module_graph = ModuleGraph::from_crate_root(&crate_root)?;

        Ok(Self {
            module_graph,
            max_depth,
            max_items_per_module,
        })
    }

    pub fn explore(&self) -> Explorer {
        Explorer::new(&self.module_graph, self.max_depth, self.max_items_per_module)
    }

    pub fn module_count(&self) -> usize {
        self.module_graph.modules().count()
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
"#,
        )
        .unwrap();

        temp_dir
    }

    #[test]
    fn test_navigator_creation() {
        let temp_dir = create_test_crate();
        let navigator = Navigator::new(temp_dir.path(), 10, 50);
        assert!(navigator.is_ok());
    }

    #[test]
    fn test_navigator_module_count() {
        let temp_dir = create_test_crate();
        let navigator = Navigator::new(temp_dir.path(), 10, 50).unwrap();
        assert_eq!(navigator.module_count(), 1);
    }

    #[test]
    fn test_navigator_explore() {
        let temp_dir = create_test_crate();
        let navigator = Navigator::new(temp_dir.path(), 10, 50).unwrap();
        let _explorer = navigator.explore();
        // Explorer functionality tested separately
    }
}
