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

    let c_fullpath_files: Vec<PathBuf> = source_files
        .iter()
        .filter(|e| return e.ends_with(".c"))
        .map(|e| return dir.join(e))
        .collect();

    let cpp_fullpath_files: Vec<PathBuf> = source_files
        .iter()
        .filter(|e| return e.ends_with(".cc"))
        .map(|e| return dir.join(e))
        .collect();

    cc::Build::new()
        .include(&dir)
        .files(c_fullpath_files)
        .warnings(false)
        .compile(name);

    if !cpp_fullpath_files.is_empty() {
        cc::Build::new()
            .cpp(true)
            .include(&dir)
            .files(cpp_fullpath_files)
            .warnings(false)
            .flag_if_supported("-std=c++14")
            .compile(format!("{}-cpp", &name).as_str());
    }

    return name.replace("tree-sitter-", "");
}

fn main() {
    vergen(Config::default()).unwrap();

    let mut langs = vec![
        build_treesitter_grammar(
            "tree-sitter-astro",
            "tree-sitter-astro/src",
            vec!["parser.c", "scanner.cc"],
        ),
        build_treesitter_grammar(
            "tree-sitter-bash",
            "tree-sitter-bash/src",
            vec!["parser.c", "scanner.cc"],
        ),
        build_treesitter_grammar(
            "tree-sitter-css",
            "tree-sitter-css/src",
            vec!["parser.c", "scanner.c"],
        ),
        build_treesitter_grammar(
            "tree-sitter-dockerfile",
            "tree-sitter-dockerfile/src",
            vec!["parser.c"],
        ),
        build_treesitter_grammar("tree-sitter-go", "tree-sitter-go/src", vec!["parser.c"]),
        build_treesitter_grammar(
            "tree-sitter-hcl",
            "tree-sitter-hcl/src",
            vec!["parser.c", "scanner.cc"],
        ),
        build_treesitter_grammar(
            "tree-sitter-html",
            "tree-sitter-html/src",
            vec!["parser.c", "scanner.cc"],
        ),
        build_treesitter_grammar(
            "tree-sitter-javascript",
            "tree-sitter-javascript/src",
            vec!["parser.c", "scanner.c"],
        ),
        "jsx".to_string(),
        build_treesitter_grammar(
            "tree-sitter-lua",
            "tree-sitter-lua/src",
            vec!["parser.c", "scanner.c"],
        ),
        build_treesitter_grammar("tree-sitter-make", "tree-sitter-make/src", vec!["parser.c"]),
        build_treesitter_grammar(
            "tree-sitter-sql",
            "tree-sitter-sql/src",
            vec!["parser.c", "scanner.cc"],
        ),
        build_treesitter_grammar(
            "tree-sitter-toml",
            "tree-sitter-toml/src",
            vec!["parser.c", "scanner.c"],
        ),
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
            "tree-sitter-python",
            "tree-sitter-python/src",
            vec!["parser.c", "scanner.cc"],
        ),
        build_treesitter_grammar(
            "tree-sitter-rust",
            "tree-sitter-rust/src",
            vec!["parser.c", "scanner.c"],
        ),
        build_treesitter_grammar(
            "tree-sitter-yaml",
            "tree-sitter-yaml/src",
            vec!["parser.c", "scanner.cc"],
        ),
    ];
    langs.sort();
    let langs_str = langs.join(",");

    println!("cargo:rustc-env=LTCC_LANGS={langs_str}");
}
