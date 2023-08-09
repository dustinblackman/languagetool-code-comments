#[cfg(test)]
#[path = "commands_test.rs"]
mod tests;

use std::collections::HashMap;

use anyhow::Result;
use languagetool_rust::check::Match;

use crate::cache;
use crate::lt;
use crate::parse;

pub async fn check(
    filepath: String,
    languagetool_api_url: String,
    concurrency: usize,
    language: String,
) -> Result<Vec<Match>> {
    let cached_match_map = cache::get_cached_matches(&filepath).await?;
    let code_comments = parse::parse_code_comments(&filepath).await?;

    // Creates a hashmap with checksums of all the code comments. This is later used
    // to dedupe requests to LanguageTool for codebases that have reoccuring
    // comments in the same file.
    let mut comments_checksum_map: HashMap<u64, String> = HashMap::new();
    for code_comment in code_comments.iter() {
        comments_checksum_map.insert(code_comment.text_checksum, code_comment.text.to_owned());
    }

    let mut result_match_map: HashMap<u64, Vec<Match>> = HashMap::new();
    let query_requests = comments_checksum_map
        .into_iter()
        .filter(|(text_checksum, _text)| {
            if cached_match_map.contains_key(text_checksum) {
                let res = cached_match_map.get(text_checksum).unwrap();
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

    let lt_client = lt::Client::new(&languagetool_api_url)?;
    let query_results = lt_client.query_many(query_requests, concurrency).await?;
    for query_result in query_results.iter() {
        // TODO remove clone.
        result_match_map.insert(query_result.text_checksum, query_result.matches.to_owned());
    }

    cache::save_cached_matches(&filepath, &result_match_map).await?;

    let concat_matches: Vec<Match> = code_comments
        .iter()
        .flat_map(|code_comment| {
            return result_match_map
                .get(&code_comment.text_checksum)
                .unwrap()
                .clone()
                .iter_mut()
                .map(|e| {
                    let more_context = e.more_context.as_mut().unwrap();
                    more_context.line_number += code_comment.start_row;
                    more_context.line_offset += code_comment.start_column;
                    // TODO remove clone.
                    return e.to_owned();
                })
                .collect::<Vec<_>>();
        })
        .collect::<Vec<_>>();

    return Ok(concat_matches);
}
