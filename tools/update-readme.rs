//! ```cargo
//! [dependencies]
//! fstrings = "0.2.3"
//! ```

use fstrings::*;
use std::fs;
use std::io::Write;
use std::process::Command;

fn main() {
    let output_help = String::from_utf8(
        Command::new("./target/debug/languagetool-code-comments")
            .arg("--help")
            .env("NO_COLOR", "1")
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();

    let version_res = String::from_utf8(
        Command::new("./target/debug/languagetool-code-comments")
            .arg("--version")
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();
    let version = version_res.split("v").collect::<Vec<&str>>()[1];

    let mut readme = fs::read_to_string("./README.md").unwrap();
    let start_help = readme.find("<!-- command-help start -->").unwrap();
    let end_help = readme.find("<!-- command-help end -->").unwrap();
    readme.replace_range(
        start_help..end_help,
        &f!("<!-- command-help start -->\n```\n{output_help}```\n"),
    );

    let start_choco = readme.find("<!-- choco-install start -->").unwrap();
    let end_choco = readme.find("<!-- choco-install end -->").unwrap();
    readme.replace_range(
        start_choco..end_choco,
        &f!("<!-- choco-install start -->\n```sh\nchoco install languagetool-code-comments --version={version}```\n"),
    );

    let mut f = fs::File::create("./README.md").unwrap();
    f.write_all(readme.as_bytes()).unwrap();
}
