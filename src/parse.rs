#[cfg(test)]
#[path = "parse_test.rs"]
mod tests;

use std::cell::RefCell;
use std::ffi::OsStr;
use std::path::Path;

use anyhow::anyhow;
use anyhow::Result;
use async_std::fs;
use fstrings::*;
use tree_sitter::Language as TSLanguage;
use tree_sitter::Node;
use tree_sitter::Parser;
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

#[derive(Clone, Debug)]
struct CommentLeaf {
    pub text: String,
    pub row: usize,
    pub column: usize,
}

struct LanguageConfig {
    /// Tree Sitter language parser instance.
    language: TSLanguage,
    /// Some languages have other embedded (Vue, Astro, Markdown). The list is
    /// used to parse child syntaxes when conditions are met.
    in_source_languages: Vec<Languages>,
    in_source_node_kind: &'static str,
    in_source_node_kind_prefix: &'static str,
}

extern "C" {
    fn tree_sitter_astro() -> TSLanguage;
    fn tree_sitter_bash() -> TSLanguage;
    fn tree_sitter_css() -> TSLanguage;
    fn tree_sitter_dockerfile() -> TSLanguage;
    fn tree_sitter_go() -> TSLanguage;
    fn tree_sitter_hcl() -> TSLanguage;
    fn tree_sitter_html() -> TSLanguage;
    fn tree_sitter_javascript() -> TSLanguage;
    fn tree_sitter_lua() -> TSLanguage;
    fn tree_sitter_nix() -> TSLanguage;
    fn tree_sitter_make() -> TSLanguage;
    fn tree_sitter_python() -> TSLanguage;
    fn tree_sitter_rust() -> TSLanguage;
    fn tree_sitter_sql() -> TSLanguage;
    fn tree_sitter_toml() -> TSLanguage;
    fn tree_sitter_tsx() -> TSLanguage;
    fn tree_sitter_typescript() -> TSLanguage;
    fn tree_sitter_yaml() -> TSLanguage;
}

#[derive(Clone, Debug)]
enum Languages {
    Astro,
    Bash,
    Css,
    Docker,
    Go,
    Hcl,
    Html,
    Javascript,
    Lua,
    Nix,
    Make,
    Python,
    Rust,
    Sql,
    Toml,
    Tsx,
    Typescript,
    Yaml,
}

/// Returns a language configuration.
fn get_language_config(lang: Languages) -> LanguageConfig {
    match lang {
        Languages::Astro => {
            return LanguageConfig {
                language: unsafe { tree_sitter_astro() },
                in_source_languages: vec![Languages::Typescript],
                in_source_node_kind: "raw_text",
                in_source_node_kind_prefix: "---",
            }
        }
        Languages::Bash => {
            return LanguageConfig {
                language: unsafe { tree_sitter_bash() },
                in_source_languages: vec![],
                in_source_node_kind: "",
                in_source_node_kind_prefix: "",
            }
        }
        Languages::Css => {
            return LanguageConfig {
                language: unsafe { tree_sitter_css() },
                in_source_languages: vec![],
                in_source_node_kind: "",
                in_source_node_kind_prefix: "",
            }
        }
        Languages::Docker => {
            return LanguageConfig {
                language: unsafe { tree_sitter_dockerfile() },
                in_source_languages: vec![],
                in_source_node_kind: "",
                in_source_node_kind_prefix: "",
            }
        }
        Languages::Go => {
            return LanguageConfig {
                language: unsafe { tree_sitter_go() },
                in_source_languages: vec![],
                in_source_node_kind: "",
                in_source_node_kind_prefix: "",
            }
        }
        Languages::Hcl => {
            return LanguageConfig {
                language: unsafe { tree_sitter_hcl() },
                in_source_languages: vec![],
                in_source_node_kind: "",
                in_source_node_kind_prefix: "",
            }
        }
        Languages::Html => {
            return LanguageConfig {
                language: unsafe { tree_sitter_html() },
                in_source_languages: vec![],
                in_source_node_kind: "",
                in_source_node_kind_prefix: "",
            }
        }
        Languages::Javascript => {
            return LanguageConfig {
                language: unsafe { tree_sitter_javascript() },
                in_source_languages: vec![],
                in_source_node_kind: "",
                in_source_node_kind_prefix: "",
            }
        }
        Languages::Lua => {
            return LanguageConfig {
                language: unsafe { tree_sitter_lua() },
                in_source_languages: vec![],
                in_source_node_kind: "",
                in_source_node_kind_prefix: "",
            }
        }
        Languages::Nix => {
            return LanguageConfig {
                language: unsafe { tree_sitter_nix() },
                in_source_languages: vec![],
                in_source_node_kind: "",
                in_source_node_kind_prefix: "",
            }
        }
        Languages::Make => {
            return LanguageConfig {
                language: unsafe { tree_sitter_make() },
                in_source_languages: vec![],
                in_source_node_kind: "",
                in_source_node_kind_prefix: "",
            }
        }
        Languages::Python => {
            return LanguageConfig {
                language: unsafe { tree_sitter_python() },
                in_source_languages: vec![],
                in_source_node_kind: "",
                in_source_node_kind_prefix: "",
            }
        }
        Languages::Rust => {
            return LanguageConfig {
                language: unsafe { tree_sitter_rust() },
                in_source_languages: vec![],
                in_source_node_kind: "",
                in_source_node_kind_prefix: "",
            }
        }
        Languages::Sql => {
            return LanguageConfig {
                language: unsafe { tree_sitter_sql() },
                in_source_languages: vec![],
                in_source_node_kind: "",
                in_source_node_kind_prefix: "",
            }
        }
        Languages::Toml => {
            return LanguageConfig {
                language: unsafe { tree_sitter_toml() },
                in_source_languages: vec![],
                in_source_node_kind: "",
                in_source_node_kind_prefix: "",
            }
        }
        Languages::Tsx => {
            return LanguageConfig {
                language: unsafe { tree_sitter_tsx() },
                in_source_languages: vec![],
                in_source_node_kind: "",
                in_source_node_kind_prefix: "",
            }
        }
        Languages::Typescript => {
            return LanguageConfig {
                language: unsafe { tree_sitter_typescript() },
                in_source_languages: vec![],
                in_source_node_kind: "",
                in_source_node_kind_prefix: "",
            }
        }
        Languages::Yaml => {
            return LanguageConfig {
                language: unsafe { tree_sitter_yaml() },
                in_source_languages: vec![],
                in_source_node_kind: "",
                in_source_node_kind_prefix: "",
            }
        }
    }
}

/// Uses a file path and its extension to detect language.
fn get_language_from_filepath(filepath: &str) -> Result<Languages> {
    let pb = Path::new(filepath);
    let mut ext = pb.extension().and_then(OsStr::to_str).unwrap_or_default();

    if ext.is_empty() {
        ext = pb.file_name().and_then(OsStr::to_str).unwrap_or_default();
    }

    match ext {
        "astro" => {
            return Ok(Languages::Astro);
        }
        "sh" => {
            return Ok(Languages::Bash);
        }
        "css" => {
            return Ok(Languages::Css);
        }
        "go" => {
            return Ok(Languages::Go);
        }
        "html" => {
            return Ok(Languages::Html);
        }
        "js" | "jsx" => {
            return Ok(Languages::Javascript);
        }
        "lua" => {
            return Ok(Languages::Lua);
        }
        "nix" => {
            return Ok(Languages::Nix);
        }
        "toml" => {
            return Ok(Languages::Toml);
        }
        "ts" => {
            return Ok(Languages::Typescript);
        }
        "tsx" => {
            return Ok(Languages::Tsx);
        }
        "py" => {
            return Ok(Languages::Python);
        }
        "rs" => {
            return Ok(Languages::Rust);
        }
        "sql" => {
            return Ok(Languages::Sql);
        }
        "tf" => {
            return Ok(Languages::Hcl);
        }
        "yaml" | "yml" => {
            return Ok(Languages::Yaml);
        }
        "Dockerfile" => {
            return Ok(Languages::Docker);
        }
        "Makefile" => {
            return Ok(Languages::Make);
        }
        _ => {
            return Err(anyhow!(f!("Can not detect language for file {filepath}")));
        }
    }
}

/// Parses the provided node searching for CommentLeafs, or further nodes to
/// scan.
fn parse_tree<'a>(
    language_config: &LanguageConfig,
    vector: &RefCell<Vec<CommentLeaf>>,
    node: Node<'a>,
    last_kind: &'static str,
    text: &'a str,
    start_row_position: usize,
    start_column_position: usize,
) -> Result<&'static str> {
    if node.child_count() == 0 {
        if node.byte_range().is_empty() {
            return Ok(node.kind());
        }

        if vec!["comment", "line_comment"].contains(&node.kind()) {
            let node_text = &text[node.byte_range()];
            let start_position = node.start_position();
            let comment_node = CommentLeaf {
                text: node_text.to_string(),
                row: start_position.row + start_row_position,
                column: start_position.column + start_column_position,
            };
            vector.borrow_mut().push(comment_node);
        }

        if !language_config.in_source_languages.is_empty()
            && node.kind() == language_config.in_source_node_kind
            && last_kind == language_config.in_source_node_kind_prefix
        {
            let node_text: &'a str = &text[node.byte_range()];
            let start_position = node.start_position();
            for lang in language_config.in_source_languages.iter() {
                let comment_nodes = get_comment_nodes_from_source(
                    lang.clone(),
                    node_text,
                    start_position.row,
                    start_position.column,
                )?;
                for e in comment_nodes.iter() {
                    vector.borrow_mut().push(e.to_owned());
                }
            }
        }

        return Ok(node.kind());
    }

    let mut cursor = node.walk();
    let mut last_kind: &'static str = "";
    for child in node.children(&mut cursor) {
        last_kind = parse_tree(
            language_config,
            vector,
            child,
            last_kind,
            text,
            start_row_position,
            start_column_position,
        )?;
    }

    return Ok("");
}

/// Kicks off parsing a Tree, appending leaves to a vector as they're found.
fn get_comment_nodes_from_source(
    lang: Languages,
    source_code: &str,
    start_row_position: usize,
    start_column_position: usize,
) -> Result<Vec<CommentLeaf>> {
    let lang_config = get_language_config(lang);
    let mut parser = Parser::new();
    parser.set_language(lang_config.language)?;

    let tree = parser.parse(source_code, None).unwrap();
    let leaves = RefCell::new(Vec::new());
    parse_tree(
        &lang_config,
        &leaves,
        tree.root_node(),
        "",
        source_code,
        start_row_position,
        start_column_position,
    )?;

    return Ok(leaves.into_inner());
}

/// Converts source code to an array of CodeComment to later be processed by
/// LanguageTool. This includes tree parsing, and hashing code comments text for
/// deduping and caching.
pub async fn parse_code_comments(filepath: &str) -> Result<Vec<CodeComment>> {
    let lang = get_language_from_filepath(filepath)?;
    let source_code = fs::read_to_string(filepath).await?;

    let code_comments = get_comment_nodes_from_source(lang, &source_code, 0, 0)?
        .iter()
        .filter(|comment_node| {
            if filepath.ends_with(".sh") && comment_node.text.starts_with("#!") {
                return false;
            }

            return true;
        })
        .map(|comment_node| {
            let text = comment_node.text.to_string();
            let text_checksum = xxh3_64(text.as_bytes());

            return CodeComment {
                text,
                text_checksum,
                start_row: comment_node.row,
                start_column: comment_node.column,
            };
        })
        .collect::<Vec<CodeComment>>();

    return Ok(code_comments);
}
