use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use super::CodeItem;

#[derive(Debug, Clone)]
pub struct Module {
    pub name: String,
    pub path: PathBuf,
    pub items: Vec<CodeItem>,
    pub submodules: Vec<String>,
    pub is_root: bool,
}

#[derive(Debug)]
pub struct ModuleGraph {
    modules: HashMap<String, Module>,
    root_module: String,
}

impl ModuleGraph {
    /// Build a module graph from a crate root
    pub fn from_crate_root(root_path: &Path) -> Result<Self> {
        let crate_dir = root_path.parent().unwrap().parent().unwrap();
        let src_dir = crate_dir.join("src");

        if !src_dir.exists() {
            anyhow::bail!("src directory not found in {:?}", crate_dir);
        }

        let mut modules = HashMap::new();
        let mut discovered_modules = HashSet::new();

        // Parse all Rust files in src/
        for entry in WalkDir::new(&src_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
        {
            let file_path = entry.path();
            let module_name = Self::path_to_module_name(&src_dir, file_path)?;

            if discovered_modules.contains(&module_name) {
                continue;
            }
            discovered_modules.insert(module_name.clone());

            let items = crate::parser::read_and_parse(file_path)?;

            // Determine if this is the root module
            let is_root = file_path
                .file_name()
                .map_or(false, |name| name == "lib.rs" || name == "main.rs");

            // Collect submodule declarations
            let submodules = Self::find_submodule_declarations(file_path)?;

            modules.insert(
                module_name.clone(),
                Module {
                    name: module_name.clone(),
                    path: file_path.to_path_buf(),
                    items,
                    submodules,
                    is_root,
                },
            );
        }

        // Find the root module
        let root_module = modules
            .iter()
            .find(|(_, m)| m.is_root)
            .map(|(name, _)| name.clone())
            .ok_or_else(|| anyhow::anyhow!("No root module found"))?;

        Ok(Self {
            modules,
            root_module,
        })
    }

    /// Get the root module
    pub fn root(&self) -> &Module {
        self.modules.get(&self.root_module).unwrap()
    }

    /// Get a module by name
    pub fn get_module(&self, name: &str) -> Option<&Module> {
        self.modules.get(name)
    }

    /// Get all modules
    pub fn modules(&self) -> impl Iterator<Item = &Module> {
        self.modules.values()
    }

    /// Get the children of a module
    pub fn children(&self, module_name: &str) -> Vec<&Module> {
        if let Some(module) = self.modules.get(module_name) {
            module
                .submodules
                .iter()
                .filter_map(|sub| {
                    let full_name = if module.is_root {
                        sub.clone()
                    } else {
                        format!("{}::{}", module_name, sub)
                    };
                    self.modules.get(&full_name)
                })
                .collect()
        } else {
            vec![]
        }
    }

    /// Convert a file path to a module name
    fn path_to_module_name(src_dir: &Path, file_path: &Path) -> Result<String> {
        let relative = file_path
            .strip_prefix(src_dir)
            .map_err(|_| anyhow::anyhow!("Path not in src dir"))?;

        let mut components = Vec::new();
        for component in relative.components() {
            let name = component.as_os_str().to_string_lossy();
            if name == "mod.rs" || name == "lib.rs" || name == "main.rs" {
                continue;
            }
            if let Some(stripped) = name.strip_suffix(".rs") {
                components.push(stripped.to_string());
            } else {
                components.push(name.to_string());
            }
        }

        if components.is_empty() {
            Ok("crate".to_string())
        } else {
            Ok(components.join("::"))
        }
    }

    /// Find module declarations in a file
    fn find_submodule_declarations(file_path: &Path) -> Result<Vec<String>> {
        let content = std::fs::read_to_string(file_path)?;
        let mut submodules = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("mod ") && !trimmed.starts_with("mod tests") {
                if let Some(name) = trimmed
                    .strip_prefix("mod ")
                    .and_then(|s| s.split(';').next())
                    .map(|s| s.trim().to_string())
                {
                    submodules.push(name);
                }
            }
        }

        Ok(submodules)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_crate(temp_dir: &Path) -> PathBuf {
        let src_dir = temp_dir.join("src");
        fs::create_dir(&src_dir).unwrap();

        // Create lib.rs
        fs::write(
            src_dir.join("lib.rs"),
            "mod foo;\npub struct RootStruct {}",
        )
        .unwrap();

        // Create foo.rs
        fs::write(src_dir.join("foo.rs"), "pub fn foo() {}").unwrap();

        src_dir.join("lib.rs")
    }

    #[test]
    fn test_module_graph_creation() {
        let temp_dir = TempDir::new().unwrap();
        let root_path = create_test_crate(temp_dir.path());

        let graph = ModuleGraph::from_crate_root(&root_path).unwrap();

        let root = graph.root();
        assert!(root.is_root);
        assert_eq!(root.name, "crate");
        assert!(root.submodules.contains(&"foo".to_string()));
    }

    #[test]
    fn test_module_children() {
        let temp_dir = TempDir::new().unwrap();
        let root_path = create_test_crate(temp_dir.path());

        let graph = ModuleGraph::from_crate_root(&root_path).unwrap();
        let children = graph.children("crate");

        assert_eq!(children.len(), 1);
        assert_eq!(children[0].name, "foo");
    }

    #[test]
    fn test_path_to_module_name() {
        let src = Path::new("/project/src");
        let file = Path::new("/project/src/foo/bar.rs");

        let name = ModuleGraph::path_to_module_name(src, file).unwrap();
        assert_eq!(name, "foo::bar");
    }
}
