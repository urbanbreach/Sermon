use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use library::open_db;
use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tauri::State;
use tracing::{info, warn};

use crate::state::ArtworkCacheState;
use crate::state::ThumbnailCacheState;

const ARTWORK_CACHE_CAP_BYTES: u64 = 256 * 1024 * 1024;
const ARTWORK_MAX_ENTRY_SIZE: u64 = ARTWORK_CACHE_CAP_BYTES / 4;

/// Common folder artwork filenames to search for (in priority order)
const FOLDER_ARTWORK_NAMES: &[&str] = &[
    "cover.jpg",
    "cover.jpeg",
    "cover.png",
    "folder.jpg",
    "folder.jpeg",
    "folder.png",
    "front.jpg",
    "front.jpeg",
    "front.png",
    "album.jpg",
    "album.jpeg",
    "album.png",
    "artwork.jpg",
    "artwork.jpeg",
    "artwork.png",
];

/// Compute cache key from album identity + provider info
pub fn compute_cache_key(
    album_artist_sort: &str,
    album_title_sort: &str,
    provider: &str,
    provider_item_id: &str,
) -> String {
    let input = format!(
        "{}||{}::{}:{}",
        album_artist_sort, album_title_sort, provider, provider_item_id
    );
    blake3::hash(input.as_bytes()).to_hex().to_string()
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtworkBytesResponse {
    pub mime: String,
    pub bytes_base64: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetBytesRequest {
    pub cache_key: String,
    pub mime: String,
}

#[tauri::command]
pub async fn cmd_artwork_get_bytes(
    request: GetBytesRequest,
    artwork_state: State<'_, ArtworkCacheState>,
) -> Result<ArtworkBytesResponse, String> {
    let _lock = artwork_state.lock.lock();

    let cache_path = artwork_state.cache_dir.join(&request.cache_key);

    if !cache_path.exists() {
        return Err(format!("Cache file not found: {}", request.cache_key));
    }

    let bytes = fs::read(&cache_path).map_err(|e| format!("Failed to read cache file: {}", e))?;
    let bytes_base64 = BASE64.encode(&bytes);

    info!(cache_key = %request.cache_key, size = bytes.len(), "artwork_cache_hit");

    Ok(ArtworkBytesResponse {
        mime: request.mime,
        bytes_base64,
    })
}

/// Write bytes to cache (used by provider fetchers and embedded extraction)
pub fn write_to_cache(
    cache_dir: &std::path::Path,
    cache_key: &str,
    bytes: &[u8],
) -> Result<(), String> {
    let entry_size = bytes.len() as u64;
    if entry_size > ARTWORK_MAX_ENTRY_SIZE {
        warn!(entry_size = entry_size, max_size = ARTWORK_MAX_ENTRY_SIZE, "artwork_cache_skip_large_entry");
        return Ok(());
    }

    let cache_path = cache_dir.join(cache_key);
    fs::write(&cache_path, bytes).map_err(|e| format!("Failed to write cache file: {}", e))?;
    info!(cache_key = %cache_key, size = bytes.len(), "artwork_cache_write");
    Ok(())
}

pub fn write_to_cache_with_eviction(
    conn: &Connection,
    cache_dir: &std::path::Path,
    cache_key: &str,
    bytes: &[u8],
) -> Result<(), String> {
    let entry_size = bytes.len() as u64;
    if entry_size > ARTWORK_MAX_ENTRY_SIZE {
        warn!(entry_size = entry_size, max_size = ARTWORK_MAX_ENTRY_SIZE, "artwork_cache_skip_large_entry");
        return Ok(());
    }

    evict_artwork_cache_lru(conn, cache_dir, entry_size);

    let cache_path = cache_dir.join(cache_key);
    fs::write(&cache_path, bytes).map_err(|e| format!("Failed to write cache file: {}", e))?;
    info!(cache_key = %cache_key, size = bytes.len(), "artwork_cache_write");
    Ok(())
}

/// Check if cache file exists
pub fn cache_exists(cache_dir: &std::path::Path, cache_key: &str) -> bool {
    cache_dir.join(cache_key).exists()
}

fn update_artwork_cache_access_album(conn: &Connection, album_artist_sort: &str, album_title_sort: &str) {
    let _ = conn.execute(
        "UPDATE artwork_cache_map_album SET selected_at = strftime('%s','now') WHERE album_artist_sort = ?1 AND album_title_sort = ?2",
        rusqlite::params![album_artist_sort, album_title_sort],
    );
}

fn update_artwork_cache_access_track(conn: &Connection, track_id: i64) {
    let _ = conn.execute(
        "UPDATE artwork_cache_map_track SET selected_at = strftime('%s','now') WHERE track_id = ?1",
        rusqlite::params![track_id],
    );
}

fn evict_artwork_cache_lru(conn: &Connection, cache_dir: &Path, target_free: u64) {
    let entries: Vec<(String, String)> = {
        let mut album_entries: Vec<(String, i64)> = conn
            .prepare("SELECT cache_key, selected_at FROM artwork_cache_map_album")
            .ok()
            .and_then(|mut stmt| {
                stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)))
                    .ok()
                    .map(|rows| rows.flatten().collect())
            })
            .unwrap_or_default();

        let mut track_entries: Vec<(String, i64)> = conn
            .prepare("SELECT cache_key, selected_at FROM artwork_cache_map_track")
            .ok()
            .and_then(|mut stmt| {
                stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)))
                    .ok()
                    .map(|rows| rows.flatten().collect())
            })
            .unwrap_or_default();

        album_entries.append(&mut track_entries);
        album_entries.sort_by_key(|(_, ts)| *ts);

        album_entries
            .into_iter()
            .map(|(key, _)| {
                let size = fs::metadata(cache_dir.join(&key))
                    .map(|m| m.len())
                    .unwrap_or(0);
                (key, size)
            })
            .filter(|(_, size)| *size > 0)
            .map(|(key, size)| (key, size.to_string()))
            .collect()
    };

    let total_size: u64 = entries.iter().map(|(_, s)| s.parse::<u64>().unwrap_or(0)).sum();
    if total_size <= ARTWORK_CACHE_CAP_BYTES.saturating_sub(target_free) {
        return;
    }

    let to_free = total_size.saturating_sub(ARTWORK_CACHE_CAP_BYTES.saturating_sub(target_free));
    let mut freed: u64 = 0;

    for (key, size_str) in entries {
        let size = size_str.parse::<u64>().unwrap_or(0);
        let cache_path = cache_dir.join(&key);
        if fs::remove_file(&cache_path).is_ok() {
            let _ = conn.execute("DELETE FROM artwork_cache_map_album WHERE cache_key = ?1", [&key]);
            let _ = conn.execute("DELETE FROM artwork_cache_map_track WHERE cache_key = ?1", [&key]);
            freed += size;
            info!(evicted_key = %key, evicted_size = size, "artwork_cache_evict");
            if freed >= to_free {
                break;
            }
        }
    }
}

// ============================================================================
// Provider Fetchers (iTunes, Deezer, fanart.tv)
// ============================================================================

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ArtworkCandidate {
    pub provider: String,
    pub provider_item_id: String,
    pub image_url: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchCandidatesResponse {
    pub candidates: Vec<ArtworkCandidate>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchCandidatesRequest {
    pub album_artist: Option<String>,
    pub album_title: Option<String>,
    pub track_title: Option<String>,
}

fn build_http_client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .connect_timeout(Duration::from_secs(3))
        .timeout(Duration::from_secs(10))
        .user_agent("Sermon/0.0.0")
        .build()
        .map_err(|e| e.to_string())
}

fn fetch_itunes_candidates(query: &str) -> Result<Vec<ArtworkCandidate>, String> {
    let client = build_http_client()?;
    let url = format!(
        "https://itunes.apple.com/search?term={}&entity=album&limit=5",
        urlencoding::encode(query)
    );

    let resp: serde_json::Value = client
        .get(&url)
        .send()
        .map_err(|e| e.to_string())?
        .json()
        .map_err(|e| e.to_string())?;

    let mut candidates = Vec::new();
    if let Some(results) = resp.get("results").and_then(|r| r.as_array()) {
        for result in results.iter().take(3) {
            if let (Some(id), Some(artwork)) = (
                result.get("collectionId").and_then(|v| v.as_i64()),
                result.get("artworkUrl100").and_then(|v| v.as_str()),
            ) {
                candidates.push(ArtworkCandidate {
                    provider: "itunes".to_string(),
                    provider_item_id: id.to_string(),
                    image_url: artwork.to_string(),
                });
            }
        }
    }
    Ok(candidates)
}

fn fetch_deezer_candidates(query: &str) -> Result<Vec<ArtworkCandidate>, String> {
    let client = build_http_client()?;
    let url = format!(
        "https://api.deezer.com/search/album?q={}&limit=5",
        urlencoding::encode(query)
    );

    let resp: serde_json::Value = client
        .get(&url)
        .send()
        .map_err(|e| e.to_string())?
        .json()
        .map_err(|e| e.to_string())?;

    let mut candidates = Vec::new();
    if let Some(data) = resp.get("data").and_then(|r| r.as_array()) {
        for item in data.iter().take(3) {
            if let (Some(id), Some(cover)) = (
                item.get("id").and_then(|v| v.as_i64()),
                item.get("cover_big").and_then(|v| v.as_str()),
            ) {
                candidates.push(ArtworkCandidate {
                    provider: "deezer".to_string(),
                    provider_item_id: id.to_string(),
                    image_url: cover.to_string(),
                });
            }
        }
    }
    Ok(candidates)
}

#[tauri::command]
pub async fn cmd_artwork_search_candidates(
    request: SearchCandidatesRequest,
    library_state: State<'_, crate::state::LibraryState>,
) -> Result<SearchCandidatesResponse, String> {
    // Snapshot mode: return fixture candidates
    if std::env::var("SERMON_SNAPSHOT").as_deref() == Ok("1") {
        info!("artwork_search_candidates snapshot_mode");
        return Ok(SearchCandidatesResponse {
            candidates: vec![
                ArtworkCandidate {
                    provider: "fixture".to_string(),
                    provider_item_id: "red".to_string(),
                    image_url: "solid-red.png".to_string(),
                },
                ArtworkCandidate {
                    provider: "fixture".to_string(),
                    provider_item_id: "blue".to_string(),
                    image_url: "solid-blue.png".to_string(),
                },
            ],
        });
    }

    // Build search query
    let query = format!(
        "{} {}",
        request.album_artist.as_deref().unwrap_or(""),
        request.album_title.as_deref().unwrap_or("")
    )
    .trim()
    .to_string();

    if query.is_empty() {
        return Ok(SearchCandidatesResponse { candidates: vec![] });
    }

    // Check provider settings
    let conn = open_db(&library_state.db_path).map_err(|e| e.to_string())?;

    let itunes_enabled = library::db::get_setting(&conn, "artwork.provider.itunes")
        .map(|v| v.as_deref() != Some("off"))
        .unwrap_or(true);
    let deezer_enabled = library::db::get_setting(&conn, "artwork.provider.deezer")
        .map(|v| v.as_deref() != Some("off"))
        .unwrap_or(true);

    // Fetch from providers in parallel using spawn_blocking
    let query_clone = query.clone();
    let results = tauri::async_runtime::spawn_blocking(move || {
        let mut candidates = Vec::new();

        // iTunes
        if itunes_enabled {
            if let Ok(itunes) = fetch_itunes_candidates(&query_clone) {
                candidates.extend(itunes);
            }
        } else {
            info!("artwork_provider_skipped itunes reason=disabled_by_user");
        }

        // Deezer
        if deezer_enabled {
            if let Ok(deezer) = fetch_deezer_candidates(&query_clone) {
                candidates.extend(deezer);
            }
        } else {
            info!("artwork_provider_skipped deezer reason=disabled_by_user");
        }

        // fanart.tv - skip (requires API key)
        info!("artwork_provider_skipped fanart reason=api_key_required");

        candidates
    })
    .await
    .map_err(|e| e.to_string())?;

    Ok(SearchCandidatesResponse {
        candidates: results,
    })
}

// ============================================================================
// Select Candidate for Album
// ============================================================================

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectCandidateRequest {
    pub album_artist_sort: String,
    pub album_title_sort: String,
    pub provider: String,
    pub provider_item_id: String,
    pub image_url: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectCandidateResponse {
    pub cache_key: String,
    pub cache_hit: bool,
}

// ============================================================================
// Best Artwork Resolution
// ============================================================================

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetBestForAlbumRequest {
    pub album_artist_sort: String,
    pub album_title_sort: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetBestForTrackRequest {
    pub track_id: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BestArtworkResponse {
    pub source: String, // "albumSelection" | "trackOverride" | "embedded" | "cache" | "none"
    pub cache_key: Option<String>,
    pub mime: Option<String>,
}

#[tauri::command]
pub async fn cmd_artwork_get_best_for_album(
    request: GetBestForAlbumRequest,
    library_state: State<'_, crate::state::LibraryState>,
    artwork_state: State<'_, ArtworkCacheState>,
) -> Result<BestArtworkResponse, String> {
    let conn = open_db(&library_state.db_path).map_err(|e| e.to_string())?;

    // Check album mapping
    let album_result: Option<(String, String)> = conn
        .query_row(
            "SELECT cache_key, mime FROM artwork_cache_map_album 
         WHERE album_artist_sort = ?1 AND album_title_sort = ?2",
            rusqlite::params![&request.album_artist_sort, &request.album_title_sort],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(|e| e.to_string())?;

    if let Some((cache_key, mime)) = album_result {
        // Verify cache file exists
        if cache_exists(&artwork_state.cache_dir, &cache_key) {
            update_artwork_cache_access_album(&conn, &request.album_artist_sort, &request.album_title_sort);
            return Ok(BestArtworkResponse {
                source: "albumSelection".to_string(),
                cache_key: Some(cache_key),
                mime: Some(mime),
            });
        }
    }

    // Check embedded/folder artwork from a representative track in this album
    let representative_track: Option<(i64, String)> = conn
        .query_row(
            r#"SELECT id, path FROM tracks 
               WHERE LOWER(COALESCE(NULLIF(TRIM(album_artist), ''), NULLIF(TRIM(artist), ''), 'unknown artist')) = ?1
                 AND LOWER(COALESCE(NULLIF(TRIM(album), ''), 'unknown album')) = ?2
                 AND is_missing = 0
               ORDER BY disc_no, track_no, id
               LIMIT 1"#,
            rusqlite::params![&request.album_artist_sort, &request.album_title_sort],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(|e| e.to_string())?;

    if let Some((track_id, track_path)) = representative_track {
        if let Some((source, cache_key, mime)) =
            try_local_artwork(track_id, &track_path, &artwork_state.cache_dir)
        {
            return Ok(BestArtworkResponse {
                source,
                cache_key: Some(cache_key),
                mime: Some(mime),
            });
        }
    }

    Ok(BestArtworkResponse {
        source: "none".to_string(),
        cache_key: None,
        mime: None,
    })
}

#[tauri::command]
pub async fn cmd_artwork_get_best_for_track(
    request: GetBestForTrackRequest,
    library_state: State<'_, crate::state::LibraryState>,
    artwork_state: State<'_, ArtworkCacheState>,
) -> Result<BestArtworkResponse, String> {
    let conn = open_db(&library_state.db_path).map_err(|e| e.to_string())?;

    // Check track mapping first
    let track_result: Option<(String, String)> = conn
        .query_row(
            "SELECT cache_key, mime FROM artwork_cache_map_track WHERE track_id = ?1",
            rusqlite::params![request.track_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(|e| e.to_string())?;

    if let Some((cache_key, mime)) = track_result {
        if cache_exists(&artwork_state.cache_dir, &cache_key) {
            update_artwork_cache_access_track(&conn, request.track_id);
            return Ok(BestArtworkResponse {
                source: "trackOverride".to_string(),
                cache_key: Some(cache_key),
                mime: Some(mime),
            });
        }
    }

    // Get track's album info to check album mapping
    let album_info: Option<(String, String)> = conn.query_row(
        r#"SELECT 
            LOWER(COALESCE(NULLIF(TRIM(album_artist), ''), NULLIF(TRIM(artist), ''), 'unknown artist')),
            LOWER(COALESCE(NULLIF(TRIM(album), ''), 'unknown album'))
         FROM tracks WHERE id = ?1"#,
        rusqlite::params![request.track_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    ).optional().map_err(|e| e.to_string())?;

    if let Some((album_artist_sort, album_title_sort)) = album_info {
        let album_result: Option<(String, String)> = conn
            .query_row(
                "SELECT cache_key, mime FROM artwork_cache_map_album 
             WHERE album_artist_sort = ?1 AND album_title_sort = ?2",
                rusqlite::params![&album_artist_sort, &album_title_sort],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(|e| e.to_string())?;

        if let Some((cache_key, mime)) = album_result {
            if cache_exists(&artwork_state.cache_dir, &cache_key) {
                update_artwork_cache_access_album(&conn, &album_artist_sort, &album_title_sort);
                return Ok(BestArtworkResponse {
                    source: "albumSelection".to_string(),
                    cache_key: Some(cache_key),
                    mime: Some(mime),
                });
            }
        }
    }

    // Check embedded/folder artwork from this track
    let track_path: Option<String> = conn
        .query_row(
            "SELECT path FROM tracks WHERE id = ?1",
            rusqlite::params![request.track_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;

    if let Some(path) = track_path {
        if let Some((source, cache_key, mime)) =
            try_local_artwork(request.track_id, &path, &artwork_state.cache_dir)
        {
            return Ok(BestArtworkResponse {
                source,
                cache_key: Some(cache_key),
                mime: Some(mime),
            });
        }
    }

    Ok(BestArtworkResponse {
        source: "none".to_string(),
        cache_key: None,
        mime: None,
    })
}

fn infer_mime_from_url(url: &str) -> String {
    if url.ends_with(".jpg") || url.ends_with(".jpeg") {
        "image/jpeg".to_string()
    } else if url.ends_with(".png") {
        "image/png".to_string()
    } else if url.ends_with(".webp") {
        "image/webp".to_string()
    } else {
        "application/octet-stream".to_string()
    }
}

#[tauri::command]
pub async fn cmd_artwork_select_candidate_for_album(
    request: SelectCandidateRequest,
    artwork_state: State<'_, ArtworkCacheState>,
    library_state: State<'_, crate::state::LibraryState>,
) -> Result<SelectCandidateResponse, String> {
    let cache_key = compute_cache_key(
        &request.album_artist_sort,
        &request.album_title_sort,
        &request.provider,
        &request.provider_item_id,
    );

    // Check if already cached (short lock scope)
    {
        let _lock = artwork_state.lock.lock();
        if cache_exists(&artwork_state.cache_dir, &cache_key) {
            info!(cache_key = %cache_key, "artwork_cache_hit");
            return Ok(SelectCandidateResponse {
                cache_key,
                cache_hit: true,
            });
        }
    }

    // Snapshot mode: no download
    if std::env::var("SERMON_SNAPSHOT").as_deref() == Ok("1") {
        info!("artwork_select snapshot_mode skip_download");
        info!(cache_key = %cache_key, "artwork_cache_miss");
        return Ok(SelectCandidateResponse {
            cache_key,
            cache_hit: false,
        });
    }

    // Download the image (no lock held)
    let image_url = request.image_url.clone();
    let (bytes, mime) = tauri::async_runtime::spawn_blocking(move || {
        let client = build_http_client()?;
        let resp = client.get(&image_url).send().map_err(|e| e.to_string())?;
        let content_type = resp
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.split(';').next().unwrap_or(s).to_string())
            .unwrap_or_else(|| infer_mime_from_url(&image_url));
        let bytes = resp.bytes().map_err(|e| e.to_string())?;
        Ok::<_, String>((bytes.to_vec(), content_type))
    })
    .await
    .map_err(|e| e.to_string())??;

    info!(cache_key = %cache_key, size = bytes.len(), "artwork_cache_miss");

    let conn = open_db(&library_state.db_path).map_err(|e| e.to_string())?;

    {
        let _lock = artwork_state.lock.lock();
        write_to_cache_with_eviction(&conn, &artwork_state.cache_dir, &cache_key, &bytes)?;
    }

    conn.execute(
        "INSERT OR REPLACE INTO artwork_cache_map_album 
         (album_artist_sort, album_title_sort, cache_key, mime, provider, provider_item_id, selected_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, strftime('%s','now'))",
        rusqlite::params![
            request.album_artist_sort,
            request.album_title_sort,
            cache_key,
            mime,
            request.provider,
            request.provider_item_id,
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(SelectCandidateResponse {
        cache_key,
        cache_hit: false,
    })
}

// ============================================================================
// Embed Artwork to File
// ============================================================================

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmbedArtworkRequest {
    pub track_id: i64,
    pub cache_key: String,
    pub mime: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmbedArtworkResponse {
    pub success: bool,
}

#[tauri::command]
pub async fn cmd_artwork_embed_to_file(
    request: EmbedArtworkRequest,
    library_state: State<'_, crate::state::LibraryState>,
    artwork_state: State<'_, ArtworkCacheState>,
) -> Result<EmbedArtworkResponse, String> {
    use tags::{write_tags, PicturePatch, TagPatches, TagWriteOptions};

    // Get track path from DB
    let conn = open_db(&library_state.db_path).map_err(|e| e.to_string())?;

    let track_path: String = conn
        .query_row(
            "SELECT path FROM tracks WHERE id = ?1",
            rusqlite::params![request.track_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Track not found: {}", e))?;

    // Read artwork bytes from cache
    let artwork_bytes = {
        let _lock = artwork_state.lock.lock();
        let cache_path = artwork_state.cache_dir.join(&request.cache_key);
        if !cache_path.exists() {
            return Err(format!("Cache file not found: {}", request.cache_key));
        }
        fs::read(&cache_path).map_err(|e| format!("Failed to read cache file: {}", e))?
    };

    // Embed using safe write strategy
    let track_path_clone = track_path.clone();
    let mime = request.mime.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        let path = std::path::Path::new(&track_path_clone);

        // Create patches with only picture set
        let patches = TagPatches {
            picture: PicturePatch::SetCover {
                bytes: artwork_bytes,
                mime,
            },
            ..Default::default()
        };

        write_tags(path, &patches, &TagWriteOptions::new())
    })
    .await
    .map_err(|e| e.to_string())?;

    result.map_err(|e| format!("Failed to embed artwork: {}", e))?;

    info!(track_id = request.track_id, cache_key = %request.cache_key, "artwork_embedded");

    Ok(EmbedArtworkResponse { success: true })
}

// ============================================================================
// Extract Embedded Artwork from File
// ============================================================================

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractEmbeddedRequest {
    pub track_id: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractEmbeddedResponse {
    pub found: bool,
    pub cache_key: Option<String>,
    pub mime: Option<String>,
}

/// Extract embedded artwork from a track's audio file and cache it
#[tauri::command]
pub async fn cmd_artwork_extract_embedded(
    request: ExtractEmbeddedRequest,
    library_state: State<'_, crate::state::LibraryState>,
    artwork_state: State<'_, ArtworkCacheState>,
) -> Result<ExtractEmbeddedResponse, String> {
    let conn = open_db(&library_state.db_path).map_err(|e| e.to_string())?;

    // Get track path from DB
    let track_path: String = conn
        .query_row(
            "SELECT path FROM tracks WHERE id = ?1",
            rusqlite::params![request.track_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Track not found: {}", e))?;

    // Extract embedded pictures using tags crate
    let track_path_clone = track_path.clone();
    let pictures = tauri::async_runtime::spawn_blocking(move || {
        tags::read_embedded_pictures(Path::new(&track_path_clone))
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| format!("Failed to read embedded pictures: {}", e))?;

    if pictures.is_empty() {
        info!(track_id = request.track_id, "no_embedded_artwork_found");
        return Ok(ExtractEmbeddedResponse {
            found: false,
            cache_key: None,
            mime: None,
        });
    }

    // Prefer CoverFront, otherwise take the first picture
    let picture = pictures
        .iter()
        .find(|p| p.picture_type.contains("CoverFront") || p.picture_type.contains("Front"))
        .or_else(|| pictures.first())
        .unwrap();

    let mime = picture
        .mime
        .clone()
        .unwrap_or_else(|| "image/jpeg".to_string());

    // Compute cache key for embedded artwork
    let cache_key = compute_cache_key(
        &format!("track_{}", request.track_id),
        "embedded",
        "embedded",
        &format!("{}", request.track_id),
    );

    {
        let _lock = artwork_state.lock.lock();
        write_to_cache_with_eviction(&conn, &artwork_state.cache_dir, &cache_key, &picture.bytes)?;
    }

    info!(
        track_id = request.track_id,
        cache_key = %cache_key,
        size = picture.bytes.len(),
        "embedded_artwork_extracted"
    );

    Ok(ExtractEmbeddedResponse {
        found: true,
        cache_key: Some(cache_key),
        mime: Some(mime),
    })
}

// ============================================================================
// Find Folder Artwork
// ============================================================================

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FindFolderArtworkRequest {
    pub track_id: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FindFolderArtworkResponse {
    pub found: bool,
    pub cache_key: Option<String>,
    pub mime: Option<String>,
    pub filename: Option<String>,
}

/// Find artwork in the same folder as the track (cover.jpg, folder.jpg, etc.)
#[tauri::command]
pub async fn cmd_artwork_find_folder(
    request: FindFolderArtworkRequest,
    library_state: State<'_, crate::state::LibraryState>,
    artwork_state: State<'_, ArtworkCacheState>,
) -> Result<FindFolderArtworkResponse, String> {
    let conn = open_db(&library_state.db_path).map_err(|e| e.to_string())?;

    // Get track path from DB
    let track_path: String = conn
        .query_row(
            "SELECT path FROM tracks WHERE id = ?1",
            rusqlite::params![request.track_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Track not found: {}", e))?;

    let track_path = Path::new(&track_path);
    let folder = track_path
        .parent()
        .ok_or_else(|| "Could not get parent folder".to_string())?;

    // Search for artwork files in priority order
    for filename in FOLDER_ARTWORK_NAMES {
        let artwork_path = folder.join(filename);
        if artwork_path.exists() && artwork_path.is_file() {
            // Found artwork file - read and cache it
            let bytes = fs::read(&artwork_path)
                .map_err(|e| format!("Failed to read artwork file: {}", e))?;

            let mime = infer_mime_from_filename(filename);

            // Compute cache key based on folder path + filename
            let folder_str = folder.to_string_lossy();
            let cache_key = compute_cache_key(&folder_str, filename, "folder", filename);

            {
                let _lock = artwork_state.lock.lock();
                if !cache_exists(&artwork_state.cache_dir, &cache_key) {
                    write_to_cache_with_eviction(&conn, &artwork_state.cache_dir, &cache_key, &bytes)?;
                }
            }

            info!(
                track_id = request.track_id,
                filename = filename,
                cache_key = %cache_key,
                "folder_artwork_found"
            );

            return Ok(FindFolderArtworkResponse {
                found: true,
                cache_key: Some(cache_key),
                mime: Some(mime),
                filename: Some(filename.to_string()),
            });
        }
    }

    info!(track_id = request.track_id, "no_folder_artwork_found");

    Ok(FindFolderArtworkResponse {
        found: false,
        cache_key: None,
        mime: None,
        filename: None,
    })
}

fn infer_mime_from_filename(filename: &str) -> String {
    let lower = filename.to_lowercase();
    if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        "image/jpeg".to_string()
    } else if lower.ends_with(".png") {
        "image/png".to_string()
    } else if lower.ends_with(".webp") {
        "image/webp".to_string()
    } else if lower.ends_with(".gif") {
        "image/gif".to_string()
    } else {
        "application/octet-stream".to_string()
    }
}

// ============================================================================
// Helper: Try to extract/find local artwork for a track
// ============================================================================

/// Try to get artwork from embedded or folder sources for a track.
/// Returns (source, cache_key, mime) if found.
pub fn try_local_artwork(
    track_id: i64,
    track_path: &str,
    cache_dir: &std::path::Path,
) -> Option<(String, String, String)> {
    let path = Path::new(track_path);

    // 1. Try embedded artwork
    if let Ok(pictures) = tags::read_embedded_pictures(path) {
        if let Some(picture) = pictures
            .iter()
            .find(|p| p.picture_type.contains("CoverFront") || p.picture_type.contains("Front"))
            .or_else(|| pictures.first())
        {
            let mime = picture
                .mime
                .clone()
                .unwrap_or_else(|| "image/jpeg".to_string());

            let meta = std::fs::metadata(path).ok()?;
            let mtime = meta
                .modified()
                .ok()?
                .duration_since(std::time::UNIX_EPOCH)
                .ok()?
                .as_millis();
            let size = meta.len();
            let provider_item_id = format!("{}:{}:{}", track_path, mtime, size);
            let cache_key = compute_cache_key("embedded", "embedded", "embedded", &provider_item_id);

            // Write to cache if not exists
            if !cache_exists(cache_dir, &cache_key) {
                if write_to_cache(cache_dir, &cache_key, &picture.bytes).is_err() {
                    return None;
                }
            }

            return Some(("embedded".to_string(), cache_key, mime));
        }
    }

    // 2. Try folder artwork
    if let Some(folder) = path.parent() {
        for filename in FOLDER_ARTWORK_NAMES {
            let artwork_path = folder.join(filename);
            if artwork_path.exists() && artwork_path.is_file() {
                if let Ok(bytes) = fs::read(&artwork_path) {
                    let mime = infer_mime_from_filename(filename);
                    let meta = match fs::metadata(&artwork_path) {
                        Ok(meta) => meta,
                        Err(_) => continue,
                    };
                    let mtime = match meta.modified() {
                        Ok(time) => match time.duration_since(std::time::UNIX_EPOCH) {
                            Ok(duration) => duration.as_millis(),
                            Err(_) => continue,
                        },
                        Err(_) => continue,
                    };
                    let size = meta.len();
                    let provider_item_id =
                        format!("{}:{}:{}", artwork_path.to_string_lossy(), mtime, size);
                    let cache_key = compute_cache_key("folder", "folder", "folder", &provider_item_id);

                    // Write to cache if not exists
                    if !cache_exists(cache_dir, &cache_key) {
                        if write_to_cache(cache_dir, &cache_key, &bytes).is_err() {
                            continue;
                        }
                    }

                    return Some(("folder".to_string(), cache_key, mime));
                }
            }
        }
    }

    None
}

// ============================================================================
// Thumbnail Cache Generation
// ============================================================================

const ALLOWED_THUMB_SIZES: [u32; 4] = [32, 128, 256, 512];
const THUMB_JPEG_QUALITY: u8 = 80;

pub fn normalize_thumbnail_size(requested: u32) -> u32 {
    ALLOWED_THUMB_SIZES
        .iter()
        .min_by_key(|&&s| (s as i32 - requested as i32).abs())
        .copied()
        .unwrap_or(256)
}

pub fn evict_thumbnail_cache_lru(cache_dir: &Path, cap_bytes: u64, target_free: u64) {
    let entries: Vec<(PathBuf, u64, std::time::SystemTime)> = match fs::read_dir(cache_dir) {
        Ok(dir) => dir
            .filter_map(|e| e.ok())
            .filter_map(|entry| {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("jpg") {
                    let meta = fs::metadata(&path).ok()?;
                    let accessed = meta.accessed().or_else(|_| meta.modified()).ok()?;
                    Some((path, meta.len(), accessed))
                } else {
                    None
                }
            })
            .collect(),
        Err(_) => return,
    };

    let total_size: u64 = entries.iter().map(|(_, size, _)| size).sum();
    if total_size <= cap_bytes.saturating_sub(target_free) {
        return;
    }

    let to_free = total_size.saturating_sub(cap_bytes.saturating_sub(target_free));
    let mut freed: u64 = 0;

    let mut sorted_entries = entries;
    sorted_entries.sort_by_key(|(_, _, accessed)| *accessed);

    for (path, size, _) in sorted_entries {
        if fs::remove_file(&path).is_ok() {
            freed += size;
            info!(evicted_path = %path.display(), evicted_size = size, "thumbnail_cache_evict");
            if freed >= to_free {
                break;
            }
        }
    }
}

pub fn generate_thumbnail(
    artwork_cache_dir: &Path,
    thumb_cache_dir: &Path,
    thumb_lock: &parking_lot::Mutex<()>,
    thumb_cap_bytes: u64,
    cache_key: &str,
    requested_size: u32,
) -> Result<PathBuf, String> {
    use image::codecs::jpeg::JpegEncoder;
    use image::imageops::FilterType;
    use image::{DynamicImage, ImageReader};

    let size = normalize_thumbnail_size(requested_size);
    let thumb_filename = format!("{}_{}.jpg", cache_key, size);
    let thumb_path = thumb_cache_dir.join(&thumb_filename);

    {
        let _lock = thumb_lock.lock();
        if thumb_path.exists() {
            return Ok(thumb_path);
        }
    }

    let source_path = artwork_cache_dir.join(cache_key);
    if !source_path.exists() {
        return Err(format!("Source artwork not found: {}", cache_key));
    }

    let source_bytes = fs::read(&source_path)
        .map_err(|e| format!("Failed to read source artwork: {}", e))?;

    let img = ImageReader::new(Cursor::new(&source_bytes))
        .with_guessed_format()
        .map_err(|e| format!("Failed to guess image format: {}", e))?
        .decode()
        .map_err(|e| format!("Failed to decode image: {}", e))?;

    let (filter, quality) = if size <= 48 {
        (FilterType::Triangle, 60)
    } else {
        (FilterType::Lanczos3, THUMB_JPEG_QUALITY)
    };

    let resized = img.resize(size, size, filter);

    let rgb_image = resized.to_rgb8();

    let mut jpeg_bytes: Vec<u8> = Vec::new();
    {
        let mut encoder = JpegEncoder::new_with_quality(&mut jpeg_bytes, quality);
        encoder
            .encode_image(&rgb_image)
            .map_err(|e| format!("Failed to encode JPEG: {}", e))?;
    }

    {
        let _lock = thumb_lock.lock();

        evict_thumbnail_cache_lru(thumb_cache_dir, thumb_cap_bytes, jpeg_bytes.len() as u64);

        fs::write(&thumb_path, &jpeg_bytes)
            .map_err(|e| format!("Failed to write thumbnail: {}", e))?;
    }

    info!(
        cache_key = %cache_key,
        size = size,
        thumb_size = jpeg_bytes.len(),
        "thumbnail_generated"
    );

    Ok(thumb_path)
}
