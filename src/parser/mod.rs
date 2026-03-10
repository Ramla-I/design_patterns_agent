mod ast;
mod module_graph;

#[allow(unused_imports)]
pub use ast::{
    CodeItem, EnumDef, Field, FunctionDef, ImplBlock, SelfParam, SourceLocation, StructDef,
    TraitDef, TypeAlias, Visibility,
};
pub use module_graph::{Module, ModuleGraph, ParseFailure};

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

/// Tolerant parse: multiple fallback strategies for nightly/edition-2024 syntax
pub fn parse_file_tolerant(path: &Path, content: &str) -> Result<Vec<CodeItem>> {
    // Pass 0: Try as-is
    if let Ok(items) = parse_file(path, content) {
        return Ok(items);
    }

    // Pass 1: Strip #![feature(...)] and #![cfg_attr(...)] single lines
    let stripped1: String = content
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !(trimmed.starts_with("#![feature(") || trimmed.starts_with("#![cfg_attr("))
        })
        .collect::<Vec<_>>()
        .join("\n");
    if let Ok(items) = parse_file(path, &stripped1) {
        return Ok(items);
    }

    // Pass 2: Strip cfg_select! { ... } blocks (brace-depth tracking)
    let stripped2 = strip_cfg_select_blocks(&stripped1);
    if stripped2 != stripped1 {
        if let Ok(items) = parse_file(path, &stripped2) {
            return Ok(items);
        }
    }

    // Pass 3: Replace `unsafe extern` → `extern` (edition 2024 syntax syn can't parse)
    let stripped3 = stripped2.replace("unsafe extern", "extern");
    if stripped3 != stripped2 {
        if let Ok(items) = parse_file(path, &stripped3) {
            return Ok(items);
        }
    }

    // Pass 4: Strip all remaining inner attributes except #![doc...] and #![allow...]
    let stripped4 = strip_inner_attributes(&stripped3);
    if stripped4 != stripped3 {
        if let Ok(items) = parse_file(path, &stripped4) {
            return Ok(items);
        }
    }

    // Pass 5: Item-level fallback — parse each top-level item individually
    if let Some(items) = parse_items_individually(path, &stripped4) {
        if !items.is_empty() {
            return Ok(items);
        }
    }

    // All passes failed, return the original error
    parse_file(path, content)
}

/// Strip `cfg_select! { ... }` blocks by tracking brace depth
fn strip_cfg_select_blocks(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let mut lines = content.lines().peekable();

    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        if trimmed.starts_with("cfg_select!") || trimmed.starts_with("cfg_select !") {
            // Count opening braces on this line
            let mut depth: i32 = 0;
            for ch in line.chars() {
                match ch {
                    '{' => depth += 1,
                    '}' => depth -= 1,
                    _ => {}
                }
            }
            // If block is fully closed on this line, skip just this line
            if depth <= 0 {
                continue;
            }
            // Otherwise, consume lines until braces balance
            while depth > 0 {
                if let Some(next_line) = lines.next() {
                    for ch in next_line.chars() {
                        match ch {
                            '{' => depth += 1,
                            '}' => depth -= 1,
                            _ => {}
                        }
                    }
                } else {
                    break;
                }
            }
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }

    result
}

/// Strip inner attributes except #![doc...] and #![allow...]
fn strip_inner_attributes(content: &str) -> String {
    content
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with("#![") {
                // Keep #![doc...] and #![allow...]
                trimmed.starts_with("#![doc") || trimmed.starts_with("#![allow")
            } else {
                true
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Parse each top-level item individually, collecting successes
fn parse_items_individually(path: &Path, content: &str) -> Option<Vec<CodeItem>> {
    let chunks = split_top_level_items(content);
    if chunks.is_empty() {
        return None;
    }

    let mut all_items = Vec::new();
    for chunk in &chunks {
        if let Ok(item) = syn::parse_str::<syn::Item>(chunk) {
            let items = ast::extract_items_from_list(&[item], path);
            all_items.extend(items);
        }
        // Skip items that fail to parse
    }

    Some(all_items)
}

/// Split source into top-level items by tracking brace depth.
/// A depth-0 `}` marks the end of an item.
fn split_top_level_items(content: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut current = String::new();
    let mut depth: i32 = 0;
    let mut in_string = false;
    let mut in_char = false;
    let mut in_line_comment = false;
    let mut in_block_comment = false;
    let mut prev_char = '\0';

    for ch in content.chars() {
        current.push(ch);

        if in_line_comment {
            if ch == '\n' {
                in_line_comment = false;
            }
            prev_char = ch;
            continue;
        }

        if in_block_comment {
            if prev_char == '*' && ch == '/' {
                in_block_comment = false;
            }
            prev_char = ch;
            continue;
        }

        if in_string {
            if ch == '"' && prev_char != '\\' {
                in_string = false;
            }
            prev_char = ch;
            continue;
        }

        if in_char {
            if ch == '\'' && prev_char != '\\' {
                in_char = false;
            }
            prev_char = ch;
            continue;
        }

        match ch {
            '/' if prev_char == '/' => {
                in_line_comment = true;
            }
            '*' if prev_char == '/' => {
                in_block_comment = true;
            }
            '"' => {
                in_string = true;
            }
            '\'' => {
                in_char = true;
            }
            '{' => {
                depth += 1;
            }
            '}' => {
                depth -= 1;
                if depth == 0 {
                    let trimmed = current.trim().to_string();
                    if !trimmed.is_empty() {
                        items.push(trimmed);
                    }
                    current.clear();
                }
            }
            _ => {}
        }

        prev_char = ch;
    }

    // Don't forget any trailing content (e.g., `use` statements, type aliases without braces)
    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() {
        // This might contain multiple simple items (use, type alias, etc.)
        // Try to split by semicolons at depth 0
        for part in trimmed.split(';') {
            let part = part.trim();
            if !part.is_empty() {
                items.push(format!("{};", part));
            }
        }
    }

    items
}

/// Discover crates in a directory (for multi-crate workspaces like rust stdlib library/)
pub fn find_workspace_crates(dir: &Path) -> Result<Vec<(String, PathBuf)>> {
    let mut crates = Vec::new();

    // Walk immediate subdirectories
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let crate_name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        // Skip hidden dirs and common non-crate dirs
        if crate_name.starts_with('.') || crate_name == "target" {
            continue;
        }

        // Look for a crate root
        if let Ok(root) = find_crate_root(&path) {
            crates.push((crate_name, root));
        }
    }

    // Sort for deterministic order
    crates.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(crates)
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

    #[test]
    fn test_parse_cfg_select_stripping() {
        let code = r#"
cfg_select! {
    feature = "nightly" => {
        fn nightly_fn() {}
    }
    _ => {
        fn stable_fn() {}
    }
}

pub struct Foo {
    pub x: i32,
}
"#;
        let items = parse_file_tolerant(Path::new("test.rs"), code).unwrap();
        assert!(items.iter().any(|item| matches!(item, CodeItem::Struct(s) if s.name == "Foo")));
    }

    #[test]
    fn test_parse_unsafe_extern() {
        let code = r#"
unsafe extern "C" {
    fn foo();
    fn bar(x: i32) -> i32;
}

pub struct Baz {
    pub y: u32,
}
"#;
        let items = parse_file_tolerant(Path::new("test.rs"), code).unwrap();
        assert!(items.iter().any(|item| matches!(item, CodeItem::Struct(s) if s.name == "Baz")));
    }

    #[test]
    fn test_parse_item_level_fallback() {
        // Source where top-level parse fails but individual items succeed
        let code = r#"
UNPARSEABLE_SYNTAX_HERE! { weird stuff }

pub struct Good {
    pub field: i32,
}

pub fn also_good() -> bool {
    true
}
"#;
        let items = parse_file_tolerant(Path::new("test.rs"), code).unwrap();
        // Should recover at least the struct and function
        assert!(items.iter().any(|item| matches!(item, CodeItem::Struct(s) if s.name == "Good")));
    }

    #[test]
    fn test_strip_inner_attributes() {
        let code = r#"
#![feature(cfg_select)]
#![allow(unused)]
#![doc = "module doc"]
#![no_std]

pub struct Foo {
    pub x: i32,
}
"#;
        // After stripping, #![feature...] and #![no_std] should be gone
        // but #![allow...] and #![doc...] should remain
        let stripped = super::strip_inner_attributes(code);
        assert!(!stripped.contains("#![feature"));
        assert!(!stripped.contains("#![no_std"));
        assert!(stripped.contains("#![allow"));
        assert!(stripped.contains("#![doc"));
    }
}
