use crate::{lt, parse};
use anyhow::Result;
use async_std::fs;
use fstrings::*;
use languagetool_rust::check::Match;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use xxhash_rust::xxh3::xxh3_64;

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
struct CacheFile {
    path: String,
    matches: HashMap<u64, Vec<Match>>,
}

fn get_cache_path() -> PathBuf {
    return dirs::config_dir()
        .unwrap()
        .join("language-tool-code-comments/cache");
}

pub async fn check(filepath: String, concurrency: usize, language: String) -> Result<()> {
    let cache_path = get_cache_path();
    if !cache_path.is_dir() {
        fs::create_dir_all(cache_path).await?;
    }
    let filepath_hash = xxh3_64(filepath.as_bytes()).to_string();
    let filepath_cache = get_cache_path().join(f!("{filepath_hash}.json"));

    // TODO move this cache stuff to somewhere else.
    let mut cached_match_map: HashMap<u64, Vec<Match>> = HashMap::new();
    if filepath_cache.is_file() {
        let cache = fs::read_to_string(filepath_cache.clone()).await?;
        let parsed: CacheFile = serde_json::from_str(&cache)?;
        cached_match_map = parsed.matches;
    }

    let code_comments = parse::parse_code_comments(&filepath).await?;

    // This is new one that'll be populated.
    // TODO better comment.
    let mut result_match_map: HashMap<u64, Vec<Match>> = HashMap::new();

    // Creates a hashmap with checksums of all the code comments. This is later used to dedupe
    // requests to LanguageTool for codebases that have reoccuring comments in the same file.
    let mut text_checksum_map: HashMap<u64, String> = HashMap::new();
    for code_comment in code_comments.iter() {
        text_checksum_map.insert(code_comment.text_checksum, code_comment.text.to_owned());
    }

    let query_requests = text_checksum_map
        .into_iter()
        .filter(|(text_checksum, _text)| {
            if cached_match_map.contains_key(text_checksum) {
                let res = cached_match_map.get(text_checksum).unwrap();
                // TODO remove clone?
                result_match_map.insert(text_checksum.to_owned(), res.to_vec());
                return false;
            }

            return true;
        })
        .map(|(text_checksum, text)| {
            return lt::QueryRequest {
                language: language.to_owned(),
                text,
                text_checksum,
            };
        })
        .collect();

    let query_results = lt::query_many(query_requests, concurrency).await?;
    for query_result in query_results.iter() {
        // TODO remove clone.
        result_match_map.insert(query_result.text_checksum, query_result.matches.to_owned());
    }

    let cache_file = CacheFile {
        path: filepath,
        matches: result_match_map.clone(),
    };
    fs::write(filepath_cache, serde_json::to_string(&cache_file)?).await?;

    let concat_matches: Vec<Match> = code_comments
        .iter()
        .flat_map(|code_comment| {
            return result_match_map
                .get(&code_comment.text_checksum)
                .unwrap()
                .clone()
                .iter_mut()
                .map(|e| {
                    let mut more_context = e.more_context.as_mut().unwrap();
                    more_context.line_number += code_comment.start_row;
                    more_context.line_offset += code_comment.start_column;
                    // TODO remove clone.
                    return e.to_owned();
                })
                .collect::<Vec<_>>();
        })
        .collect::<Vec<_>>();

    println!("{}", serde_json::to_string(&concat_matches)?);

    return Ok(());
}
