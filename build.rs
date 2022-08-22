#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]

use std::path::PathBuf;
use vergen::{vergen, Config};

fn build_treesitter_grammar(
    name: &str,
    relative_source_path: &str,
    source_files: Vec<&str>,
) -> String {
    let dir: PathBuf = ["external"]
        .iter()
        .collect::<PathBuf>()
        .join(relative_source_path);

    let mut build = cc::Build::new();
    build.include(&dir);
    for source_file in source_files.iter() {
        build.file(dir.join(source_file));
    }
    build.compile(name);

    return name.replace("tree-sitter-", "");
}

fn main() {
    vergen(Config::default()).unwrap();

    let mut langs = vec![
        build_treesitter_grammar(
            "tree-sitter-typescript",
            "tree-sitter-typescript/typescript/src",
            vec!["parser.c", "scanner.c"],
        ),
        build_treesitter_grammar(
            "tree-sitter-tsx",
            "tree-sitter-typescript/tsx/src",
            vec!["parser.c", "scanner.c"],
        ),
        build_treesitter_grammar(
            "tree-sitter-rust",
            "tree-sitter-rust/src",
            vec!["parser.c", "scanner.c"],
        ),
    ];
    langs.sort();
    let langs_str = langs.join(",");

    println!("cargo:rustc-env=LTCC_LANGS={langs_str}");
}
