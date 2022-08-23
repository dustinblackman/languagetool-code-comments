//! ```cargo
//! [dependencies]
//! fstrings = "0.2.3"
//! ```

use fstrings::*;
use std::fs;
use std::io::Write;
use std::process::Command;

fn main() {
    let output = String::from_utf8(
        Command::new("./target/debug/languagetool-code-comments")
            .arg("--help")
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();

    let mut readme = fs::read_to_string("./README.md").unwrap();
    let start = readme.find("<!-- command-help start -->").unwrap();
    let end = readme.find("<!-- command-help end -->").unwrap();
    readme.replace_range(
        start..end,
        &f!("<!-- command-help start -->\n```\n{output}```\n"),
    );

    let mut f = fs::File::create("./README.md").unwrap();
    f.write_all(readme.as_bytes()).unwrap();
}
