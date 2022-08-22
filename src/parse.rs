use anyhow::Result;
use std::cell::RefCell;
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
    fn tree_sitter_typescript() -> Language;
}

/// Parses the provided node searching for CommentLeafs, or further nodes to scan.
fn parse_tree<'a>(vector: &RefCell<Vec<CommentLeaf<'a>>>, node: Node<'a>, text: &'a str) {
    if node.child_count() == 0 {
        if !node.byte_range().is_empty() && node.kind() == "comment" {
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

/// Converts source code to an array of CodeComment to later be processed by LanguageTool. This
/// includes tree parsing, and hashing code comments text for deduping and caching.
pub fn parse_code_comments(source_code: String) -> Result<Vec<CodeComment>> {
    let language = unsafe { tree_sitter_typescript() };
    let mut parser = Parser::new();
    parser.set_language(language)?;
    let tree = parser.parse(source_code.clone(), None).unwrap();

    let code_comments = get_comment_nodes(&tree, &source_code)?
        .iter()
        .filter(|comment_node| {
            return comment_node.reference.kind() == "comment";
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
