use super::*;
use crate::cache;
use anyhow::Result;
use async_std::fs;
use axum::{
    // extract::Form,
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Json,
    Router,
};
use expectest::prelude::*;
use languagetool_rust::check::CheckResponse;
use std::env;
use std::net::SocketAddr;

async fn lt_check_match_results() -> impl IntoResponse {
    let data = fs::read_to_string("./tests/fixtures/commands/lt_with_matches.json")
        .await
        .unwrap();
    let res: CheckResponse = serde_json::from_str(&data).unwrap();
    return (StatusCode::OK, Json(res));
}

#[tokio::test]
async fn test_commands_happy_path() -> Result<()> {
    let http_thread = tokio::spawn(async move {
        let app = Router::new().route("/v2/check", post(lt_check_match_results));

        let addr = SocketAddr::from(([127, 0, 0, 1], 3344));
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .unwrap();
    });

    let filepath = env::current_dir()?
        .join("tests/fixtures/commands/code.py")
        .as_path()
        .display()
        .to_string();

    let cache_filepath = cache::get_file_path(&filepath).await?;
    if cache_filepath.is_file() {
        fs::remove_file(cache_filepath).await?;
    }

    let res = check(
        filepath,
        "http://127.0.0.1:3344".to_string(),
        1,
        "en".to_string(),
    )
    .await?;

    expect(res.len()).to(be_equal_to(1));

    http_thread.abort();
    return Ok(());
}
