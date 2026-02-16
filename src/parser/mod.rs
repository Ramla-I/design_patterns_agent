mod ast;
mod module_graph;

pub use ast::{
    CodeItem, EnumDef, FunctionDef, ImplBlock, SelfParam, SourceLocation, StructDef, TraitDef, TypeAlias, Visibility,
};
pub use module_graph::{Module, ModuleGraph};

use anyhow::Result;
use std::path::{Path, PathBuf};

/// Parse a single Rust file and extract code items
pub fn parse_file(path: &Path, content: &str) -> Result<Vec<CodeItem>> {
    let syntax_tree = syn::parse_file(content)?;
    Ok(ast::extract_items(&syntax_tree, path))
}

/// Find the crate root (lib.rs or main.rs) in a directory
pub fn find_crate_root(dir: &Path) -> Result<PathBuf> {
    // Check standard Rust project layout: src/lib.rs or src/main.rs
    let lib_rs = dir.join("src/lib.rs");
    if lib_rs.exists() {
        return Ok(lib_rs);
    }

    let main_rs = dir.join("src/main.rs");
    if main_rs.exists() {
        return Ok(main_rs);
    }

    // Check for lib.rs or main.rs directly in the root (e.g., translated_rust folders)
    let root_lib_rs = dir.join("lib.rs");
    if root_lib_rs.exists() {
        return Ok(root_lib_rs);
    }

    let root_main_rs = dir.join("main.rs");
    if root_main_rs.exists() {
        return Ok(root_main_rs);
    }

    anyhow::bail!("Could not find crate root (lib.rs or main.rs) in {:?}", dir)
}

/// Read and parse a Rust file
pub fn read_and_parse(path: &Path) -> Result<Vec<CodeItem>> {
    let content = std::fs::read_to_string(path)?;
    parse_file(path, &content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_parse_simple_struct() {
        let code = r#"
/// A simple struct
pub struct Foo {
    pub bar: i32,
}
"#;
        let items = parse_file(Path::new("test.rs"), code).unwrap();
        assert_eq!(items.len(), 1);
        match &items[0] {
            CodeItem::Struct(s) => {
                assert_eq!(s.name, "Foo");
                assert_eq!(s.fields.len(), 1);
                assert_eq!(s.doc_comment.as_ref().unwrap(), " A simple struct");
            }
            _ => panic!("Expected struct"),
        }
    }

    #[test]
    fn test_parse_enum() {
        let code = r#"
pub enum State {
    Open,
    Closed,
}
"#;
        let items = parse_file(Path::new("test.rs"), code).unwrap();
        assert_eq!(items.len(), 1);
        match &items[0] {
            CodeItem::Enum(e) => {
                assert_eq!(e.name, "State");
                assert_eq!(e.variants.len(), 2);
            }
            _ => panic!("Expected enum"),
        }
    }

    #[test]
    fn test_find_crate_root_lib() {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir).unwrap();
        fs::write(src_dir.join("lib.rs"), "").unwrap();

        let root = find_crate_root(temp_dir.path()).unwrap();
        assert_eq!(root, src_dir.join("lib.rs"));
    }

    #[test]
    fn test_find_crate_root_main() {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir).unwrap();
        fs::write(src_dir.join("main.rs"), "").unwrap();

        let root = find_crate_root(temp_dir.path()).unwrap();
        assert_eq!(root, src_dir.join("main.rs"));
    }
}
