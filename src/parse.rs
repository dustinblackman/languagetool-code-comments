#[cfg(test)]
#[path = "parse_test.rs"]
mod tests;

use anyhow::{anyhow, Result};
use async_std::fs;
use fstrings::*;
use std::cell::RefCell;
use std::ffi::OsStr;
use std::path::Path;
use tree_sitter::{Language, Node, Parser, Tree};
use xxhash_rust::xxh3::xxh3_64;

#[derive(Clone, Debug)]
pub struct CodeComment {
    pub text: String,
    pub text_checksum: u64,
    pub start_row: usize,
    pub start_column: usize,
}
unsafe impl Send for CodeComment {}
unsafe impl Sync for CodeComment {}

struct CommentLeaf<'a> {
    pub reference: Node<'a>,
    pub text: &'a str,
}

extern "C" {
 fn tree_sitter_bash() -> Language;
    fn tree_sitter_css() -> Language;
    fn tree_sitter_dockerfile() -> Language;
    fn tree_sitter_go() -> Language;
    fn tree_sitter_hcl() -> Language;
    fn tree_sitter_html() -> Language;
    fn tree_sitter_javascript() -> Language;
    fn tree_sitter_lua() -> Language;
    fn tree_sitter_make() -> Language;
    fn tree_sitter_python() -> Language;
    fn tree_sitter_rust() -> Language;
    fn tree_sitter_sql() -> Language;
    fn tree_sitter_toml() -> Language;
    fn tree_sitter_tsx() -> Language;
    fn tree_sitter_typescript() -> Language;
}

/// Parses the provided node searching for CommentLeafs, or further nodes to scan.
fn parse_tree<'a>(vector: &RefCell<Vec<CommentLeaf<'a>>>, node: Node<'a>, text: &'a str) {
    if node.child_count() == 0 {
        if !node.byte_range().is_empty() && vec!["comment", "line_comment"].contains(&node.kind()) {
            let node_text: &'a str = &text[node.byte_range()];
            vector.borrow_mut().push(CommentLeaf {
                reference: node,
                text: node_text,
            });
        }
        return;
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        parse_tree(vector, child, text);
    }
}

/// Kicks off parsing a Tree, appending leaves to a vector as they're found.
fn get_comment_nodes<'a>(tree: &'a Tree, source_code: &'a str) -> Result<Vec<CommentLeaf<'a>>> {
    let leaves = RefCell::new(Vec::new());
    parse_tree(&leaves, tree.root_node(), source_code);

    return Ok(leaves.into_inner());
}

/// Uses a file path, and its extension to detect language, and returns an initialized Tree Sitter.
/// parser for source code parsing.
fn get_parser(filepath: &str) -> Result<Parser> {
    let mut parser = Parser::new();
    let pb = Path::new(filepath);
    let mut ext = pb
        .extension()
        .and_then(OsStr::to_str)
        .unwrap_or_default();

    if ext.is_empty() {
        ext = pb.file_name().and_then(OsStr::to_str).unwrap_or_default();
    }

    match ext {
        "sh" => {
            parser.set_language(unsafe { tree_sitter_bash() })?;
        }
        "css" => {
            parser.set_language(unsafe { tree_sitter_css() })?;
        }
        "go" => {
            parser.set_language(unsafe { tree_sitter_go() })?;
        }
        "html" => {
            parser.set_language(unsafe { tree_sitter_html() })?;
        }
        "js" => {
            parser.set_language(unsafe { tree_sitter_javascript() })?;
        }
        "jsx" => {
            parser.set_language(unsafe { tree_sitter_javascript() })?;
        }
        "lua" => {
            parser.set_language(unsafe { tree_sitter_lua() })?;
        }
        "toml" => {
            parser.set_language(unsafe { tree_sitter_toml() })?;
        }
        "ts" => {
            parser.set_language(unsafe { tree_sitter_typescript() })?;
        }
        "tsx" => {
            parser.set_language(unsafe { tree_sitter_tsx() })?;
        }
        "py" => {
            parser.set_language(unsafe { tree_sitter_python() })?;
        }
        "rs" => {
            parser.set_language(unsafe { tree_sitter_rust() })?;
        }
        "sql" => {
            parser.set_language(unsafe { tree_sitter_sql() })?;
        }
        "tf" => {
            parser.set_language(unsafe { tree_sitter_hcl() })?;
        }
        "Dockerfile" => {
            parser.set_language(unsafe { tree_sitter_dockerfile() })?;
        }
        "Makefile" => {
            parser.set_language(unsafe { tree_sitter_make() })?;
        }
        _ => {
            return Err(anyhow!(f!("Can not detect language for file {filepath}")));
        }
    }

    return Ok(parser);
}

/// Converts source code to an array of CodeComment to later be processed by LanguageTool. This
/// includes tree parsing, and hashing code comments text for deduping and caching.
pub async fn parse_code_comments(filepath: &str) -> Result<Vec<CodeComment>> {
    let mut parser = get_parser(filepath)?;
    let source_code = fs::read_to_string(filepath).await?;
    let tree = parser.parse(source_code.clone(), None).unwrap();

    let code_comments = get_comment_nodes(&tree, &source_code)?
        .iter()
        .filter(|comment_node| {
            if filepath.ends_with(".sh") && comment_node.text.starts_with("#!") {
                return false;
            }

            return true;
        })
        .map(|comment_node| {
            let start_position = comment_node.reference.start_position();
            let text = comment_node.text.to_string();
            let text_checksum = xxh3_64(text.as_bytes());

            return CodeComment {
                text,
                text_checksum,
                start_row: start_position.row,
                start_column: start_position.column,
            };
        })
        .collect::<Vec<CodeComment>>();

    return Ok(code_comments);
}
