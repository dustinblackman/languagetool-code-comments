use std::path::PathBuf;
use vergen::{vergen, Config};

fn build_treesitter_language(name: &str, relative_source_path: &str, source_files: Vec<&str>) {
    let dir: PathBuf = ["external", name]
        .iter()
        .collect::<PathBuf>()
        .join(relative_source_path);

    let mut build = cc::Build::new();
    build.include(&dir);
    for source_file in source_files.iter() {
        build.file(dir.join(source_file));
    }
    build.compile(name);
}

fn main() {
    // vergen(Config::default()).unwrap();

    // TODO add build env listening all the available languages.
    // println!("cargo:rerun-if-env-changed=LTCC_LANGS");
    // println!("cargo:rustc-env=LTCC_LANGS={bin_name}");

    build_treesitter_language(
        "tree-sitter-typescript",
        "typescript/src",
        vec!["parser.c", "scanner.c"],
    );

    // let dir: PathBuf = ["tree-sitter-typescript", "typescript", "src"]
    // .iter()
    // .collect();

    // // TODO need to setup function for this.
    // cc::Build::new()
    // .include(&dir)
    // .file(dir.join("parser.c"))
    // .file(dir.join("scanner.c"))
    // .compile("tree-sitter-typescript");
}
