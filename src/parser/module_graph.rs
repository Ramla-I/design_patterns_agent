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

#[derive(Debug, Clone)]
pub struct ParseFailure {
    pub file_path: String,
    pub error: String,
}

#[derive(Debug)]
pub struct ModuleGraph {
    modules: HashMap<String, Module>,
    root_module: String,
    pub parse_failures: Vec<ParseFailure>,
}

impl ModuleGraph {
    /// Build a module graph from a crate root
    pub fn from_crate_root(root_path: &Path) -> Result<Self> {
        Self::from_crate_root_with_prefix(root_path, None)
    }

    /// Build a module graph from a crate root, optionally prefixing module names
    fn from_crate_root_with_prefix(root_path: &Path, crate_prefix: Option<&str>) -> Result<Self> {
        let root_parent = root_path.parent().unwrap();
        let is_standard_layout = root_parent.file_name().map_or(false, |n| n == "src");
        let src_dir = root_parent.to_path_buf();

        let mut modules = HashMap::new();
        let mut discovered_modules = HashSet::new();
        let mut parse_failures = Vec::new();

        let mut dirs_to_scan = vec![src_dir.clone()];

        if !is_standard_layout {
            let potential_src = root_parent.join("src");
            if potential_src.exists() && potential_src.is_dir() {
                dirs_to_scan.push(potential_src);
            }
        }

        for scan_dir in &dirs_to_scan {
            for entry in WalkDir::new(scan_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
                .filter(|e| !e.path().to_string_lossy().contains("/target/"))
            {
                let file_path = entry.path();
                let raw_module_name = match Self::path_to_module_name(&src_dir, file_path) {
                    Ok(name) => name,
                    Err(_) => continue,
                };

                // Apply crate prefix: "crate" -> "std", "sync::mutex" -> "std::sync::mutex"
                let module_name = if let Some(prefix) = crate_prefix {
                    if raw_module_name == "crate" {
                        prefix.to_string()
                    } else {
                        format!("{}::{}", prefix, raw_module_name)
                    }
                } else {
                    raw_module_name
                };

                if discovered_modules.contains(&module_name) {
                    continue;
                }
                discovered_modules.insert(module_name.clone());

                // Tolerant parse: try normal first, then strip feature gates
                let content = match std::fs::read_to_string(file_path) {
                    Ok(c) => c,
                    Err(e) => {
                        parse_failures.push(ParseFailure {
                            file_path: file_path.to_string_lossy().to_string(),
                            error: format!("read error: {}", e),
                        });
                        continue;
                    }
                };

                let items = match crate::parser::parse_file_tolerant(file_path, &content) {
                    Ok(items) => items,
                    Err(e) => {
                        let err_msg = format!("{}", e);
                        eprintln!("  Warning: skipping {} (syn parse error: {})", file_path.display(), err_msg);
                        parse_failures.push(ParseFailure {
                            file_path: file_path.to_string_lossy().to_string(),
                            error: err_msg,
                        });
                        vec![]
                    }
                };

                let is_root = file_path
                    .file_name()
                    .map_or(false, |name| name == "lib.rs" || name == "main.rs");

                let submodules = Self::find_submodule_declarations(file_path).unwrap_or_default();

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
        }

        let root_module = modules
            .iter()
            .find(|(_, m)| m.is_root)
            .map(|(name, _)| name.clone())
            .ok_or_else(|| anyhow::anyhow!("No root module found"))?;

        Ok(Self {
            modules,
            root_module,
            parse_failures,
        })
    }

    /// Build a merged module graph from multiple crate roots (multi-crate workspace)
    pub fn from_workspace(crate_roots: Vec<(String, PathBuf)>) -> Result<Self> {
        let mut all_modules = HashMap::new();
        let mut all_failures = Vec::new();
        let mut root_names = Vec::new();

        for (crate_name, root_path) in &crate_roots {
            match Self::from_crate_root_with_prefix(root_path, Some(crate_name)) {
                Ok(graph) => {
                    root_names.push(crate_name.clone());
                    all_failures.extend(graph.parse_failures);
                    for (name, module) in graph.modules {
                        all_modules.insert(name, module);
                    }
                }
                Err(e) => {
                    eprintln!("  Warning: failed to parse crate {}: {}", crate_name, e);
                    all_failures.push(ParseFailure {
                        file_path: root_path.to_string_lossy().to_string(),
                        error: format!("crate parse failed: {}", e),
                    });
                }
            }
        }

        // Create a synthetic workspace root
        let workspace_root = Module {
            name: "workspace".to_string(),
            path: PathBuf::new(),
            items: vec![],
            submodules: root_names,
            is_root: true,
        };
        all_modules.insert("workspace".to_string(), workspace_root);

        Ok(Self {
            modules: all_modules,
            root_module: "workspace".to_string(),
            parse_failures: all_failures,
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

            // Skip comments
            if trimmed.starts_with("//") || trimmed.starts_with("/*") {
                continue;
            }

            // Strip visibility qualifiers to find "mod <name>"
            // Handles: mod foo; | pub mod foo; | pub(crate) mod foo; | pub(super) mod foo;
            let after_vis = if trimmed.starts_with("pub(") {
                // pub(crate) mod ..., pub(super) mod ...
                if let Some(paren_end) = trimmed.find(')') {
                    trimmed[paren_end + 1..].trim_start()
                } else {
                    continue;
                }
            } else if let Some(rest) = trimmed.strip_prefix("pub ") {
                rest.trim_start()
            } else {
                trimmed
            };

            if let Some(rest) = after_vis.strip_prefix("mod ") {
                // Extract module name from "mod <name>;", "mod <name> {", etc.
                let name = rest
                    .split(|c: char| c == ';' || c == '{' || c.is_whitespace())
                    .next()
                    .unwrap_or("")
                    .trim();

                if !name.is_empty() && name != "tests" {
                    submodules.push(name.to_string());
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

    #[test]
    fn test_pub_mod_declarations() {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir).unwrap();

        fs::write(
            src_dir.join("lib.rs"),
            "pub mod foo;\npub(crate) mod bar;\nmod baz;\n// mod commented_out;\n",
        )
        .unwrap();
        fs::write(src_dir.join("foo.rs"), "pub fn foo() {}").unwrap();
        fs::write(src_dir.join("bar.rs"), "pub fn bar() {}").unwrap();
        fs::write(src_dir.join("baz.rs"), "pub fn baz() {}").unwrap();

        let root_path = src_dir.join("lib.rs");
        let graph = ModuleGraph::from_crate_root(&root_path).unwrap();
        let root = graph.root();

        assert!(root.submodules.contains(&"foo".to_string()));
        assert!(root.submodules.contains(&"bar".to_string()));
        assert!(root.submodules.contains(&"baz".to_string()));
        assert!(!root.submodules.contains(&"commented_out".to_string()));
    }
}
