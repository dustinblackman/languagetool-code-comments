use super::*;
use anyhow::Result;
use expectest::prelude::*;

#[tokio::test]
async fn test_bash() -> Result<()> {
    let res = parse_code_comments("./tests/fixtures/bash.sh").await?;
    expect!(res.len()).to(be_equal_to(2));
    expect!(res[0].text.to_owned()).to(be_equal_to("# I am comment number one."));

    return Ok(());
}

#[tokio::test]
async fn test_css() -> Result<()> {
    let res = parse_code_comments("./tests/fixtures/css.css").await?;
    expect!(res.len()).to(be_equal_to(2));
    expect!(res[0].text.to_owned()).to(be_equal_to("/* I am comment number one. */"));

    return Ok(());
}

#[tokio::test]
async fn test_go() -> Result<()> {
    let res = parse_code_comments("./tests/fixtures/golang.go").await?;
    expect!(res.len()).to(be_equal_to(2));
    expect!(res[0].text.to_owned()).to(be_equal_to("// I am comment number one."));

    return Ok(());
}

#[tokio::test]
async fn test_hcl() -> Result<()> {
    let res = parse_code_comments("./tests/fixtures/hcl.tf").await?;
    expect!(res.len()).to(be_equal_to(2));
    expect!(res[0].text.to_owned()).to(be_equal_to("# I am comment number one."));

    return Ok(());
}

#[tokio::test]
async fn test_javascript() -> Result<()> {
    let res = parse_code_comments("./tests/fixtures/javascript.js").await?;
    expect!(res.len()).to(be_equal_to(2));
    expect!(res[0].text.to_owned()).to(be_equal_to("/**\n  * I am comment number one.\n  */"));

    return Ok(());
}

#[tokio::test]
async fn test_javascript_react() -> Result<()> {
    let res = parse_code_comments("./tests/fixtures/javascript-react.jsx").await?;
    expect!(res.len()).to(be_equal_to(2));
    expect!(res[0].text.to_owned()).to(be_equal_to("/**\n  * I am comment number one.\n  */"));

    return Ok(());
}

#[tokio::test]
async fn test_typescript() -> Result<()> {
    let res = parse_code_comments("./tests/fixtures/typescript.ts").await?;
    expect!(res.len()).to(be_equal_to(2));
    expect!(res[0].text.to_owned()).to(be_equal_to("/**\n  * I am comment number one.\n  */"));

    return Ok(());
}

#[tokio::test]
async fn test_typescript_react() -> Result<()> {
    let res = parse_code_comments("./tests/fixtures/typescript-react.tsx").await?;
    expect!(res.len()).to(be_equal_to(2));
    expect!(res[0].text.to_owned()).to(be_equal_to("/**\n  * I am comment number one.\n  */"));

    return Ok(());
}

#[tokio::test]
async fn test_python() -> Result<()> {
    let res = parse_code_comments("./tests/fixtures/python.py").await?;
    expect!(res.len()).to(be_equal_to(2));
    expect!(res[0].text.to_owned()).to(be_equal_to("# I am comment number one."));

    return Ok(());
}

#[tokio::test]
async fn test_rust() -> Result<()> {
    let res = parse_code_comments("./tests/fixtures/rust.rs").await?;
    expect!(res.len()).to(be_equal_to(2));
    expect!(res[0].text.to_owned()).to(be_equal_to("// I am comment number one."));

    return Ok(());
}

#[tokio::test]
async fn test_error() -> Result<()> {
    let res = parse_code_comments("./tests/fixtures/fail.txt").await;
    expect!(res.is_err()).to(be_equal_to(true));

    return Ok(());
}
