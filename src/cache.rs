use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
use async_std::fs;
use fstrings::*;
use languagetool_rust::check::Match;
use serde::Deserialize;
use serde::Serialize;
use xxhash_rust::xxh3::xxh3_64;

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
struct CacheFile {
    path: String,
    matches: HashMap<u64, Vec<Match>>,
}

/// Returns the cache directory for the current user.
pub async fn get_dir_path() -> Result<PathBuf> {
    let cache_dir = dirs::config_dir()
        .unwrap()
        .join("language-tool-code-comments/cache");

    if !cache_dir.is_dir() {
        fs::create_dir_all(&cache_dir).await?;
    }

    return Ok(cache_dir);
}

/// Deletes the entire cache directory.
pub async fn delete_cache() -> Result<()> {
    let cache_dir = get_dir_path().await?;
    fs::remove_dir_all(cache_dir).await?;
    return Ok(());
}

/// Returns the cache path for a source code path.
pub async fn get_file_path(filepath: &str) -> Result<PathBuf> {
    let filepath_hash = xxh3_64(filepath.as_bytes()).to_string();
    let cache_dir = get_dir_path().await?;
    return Ok(cache_dir.join(f!("{filepath_hash}.json")));
}

/// Loads the cached match entries from cache, returning an empty hashmap if a
/// cache entry did not exist.
pub async fn get_cached_matches(filepath: &str) -> Result<HashMap<u64, Vec<Match>>> {
    let filepath_cache = get_file_path(filepath).await?;

    let mut cached_match_map: HashMap<u64, Vec<Match>> = HashMap::new();
    if filepath_cache.is_file() {
        let cache = fs::read_to_string(filepath_cache.clone()).await?;
        let parsed: CacheFile = serde_json::from_str(&cache)?;
        cached_match_map = parsed.matches;
    }

    return Ok(cached_match_map);
}

/// Persists result matches to disk.
pub async fn save_cached_matches(
    filepath: &str,
    result_match_map: &HashMap<u64, Vec<Match>>,
) -> Result<()> {
    let cache_file = CacheFile {
        path: filepath.to_string(),
        matches: result_match_map.clone(),
    };
    let filepath_cache = get_file_path(filepath).await?;
    fs::write(filepath_cache, serde_json::to_string(&cache_file)?).await?;

    return Ok(());
}
