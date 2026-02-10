use library::open_db;
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::thread;
use std::time::Duration;
use tauri::State;
use tracing::warn;

const LRCLIB_BASE_URL: &str = "https://lrclib.net";
const DURATION_TOLERANCE_SECS: i64 = 3;
const RETRY_BACKOFF_MS: u64 = 500;

const SCORE_EXACT_TITLE: i32 = 10;
const SCORE_EXACT_ARTIST: i32 = 10;
const SCORE_DURATION_TOLERANCE: i32 = 5;
const SCORE_SYNCED_BONUS: i32 = 3;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LyricsRequest {
    pub track_id: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LyricsResponse {
    pub track_id: i64,
    pub synced_lyrics: Option<String>,
    pub plain_lyrics: Option<String>,
    pub lyricist: Option<String>,
    pub source: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ResolvedLyrics {
    synced_lyrics: Option<String>,
    plain_lyrics: Option<String>,
    lyricist: Option<String>,
}

impl ResolvedLyrics {
    fn has_any(&self) -> bool {
        self.synced_lyrics.is_some() || self.plain_lyrics.is_some() || self.lyricist.is_some()
    }

    fn into_response(self, track_id: i64, source: &str) -> LyricsResponse {
        LyricsResponse {
            track_id,
            synced_lyrics: self.synced_lyrics,
            plain_lyrics: self.plain_lyrics,
            lyricist: self.lyricist,
            source: source.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
struct TrackLookup {
    path: String,
    title: String,
    artist: String,
    album: String,
    duration_secs: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LrclibCandidate {
    track_name: Option<String>,
    artist_name: Option<String>,
    duration: Option<f64>,
    synced_lyrics: Option<String>,
    plain_lyrics: Option<String>,
}

#[tauri::command]
pub async fn cmd_lyrics_get_for_track(
    request: LyricsRequest,
    library_state: State<'_, crate::state::LibraryState>,
) -> Result<LyricsResponse, String> {
    let conn = open_db(&library_state.db_path).map_err(|e| e.to_string())?;

    let track = load_track_lookup(&conn, request.track_id)?
        .ok_or_else(|| format!("Track not found: {}", request.track_id))?;

    if let Some(embedded) = read_embedded_lyrics(&track.path) {
        return Ok(resolve_precedence(
            request.track_id,
            Some(embedded),
            None,
            None,
        ));
    }

    if let Some(cached_response) = load_cached_lyrics_response(&conn, request.track_id)? {
        return Ok(cached_response);
    }

    let track_for_lookup = track.clone();
    let fetched =
        tauri::async_runtime::spawn_blocking(move || fetch_lrclib_lyrics(&track_for_lookup))
            .await
            .map_err(|e| e.to_string())??;

    if let Some(ref lyrics) = fetched {
        cache_lyrics(&conn, request.track_id, lyrics)?;
    }

    Ok(resolve_precedence(request.track_id, None, None, fetched))
}

fn load_track_lookup(
    conn: &rusqlite::Connection,
    track_id: i64,
) -> Result<Option<TrackLookup>, String> {
    let row: Option<(
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<i64>,
    )> = conn
        .query_row(
            "SELECT path, title, artist, album, duration_ms FROM tracks WHERE id = ?1",
            rusqlite::params![track_id],
            |record| {
                Ok((
                    record.get(0)?,
                    record.get(1)?,
                    record.get(2)?,
                    record.get(3)?,
                    record.get(4)?,
                ))
            },
        )
        .optional()
        .map_err(|e| e.to_string())?;

    Ok(
        row.map(|(path, title, artist, album, duration_ms)| TrackLookup {
            path,
            title: title.unwrap_or_default().trim().to_string(),
            artist: artist.unwrap_or_default().trim().to_string(),
            album: album.unwrap_or_default().trim().to_string(),
            duration_secs: duration_ms_to_secs(duration_ms),
        }),
    )
}

fn read_embedded_lyrics(track_path: &str) -> Option<ResolvedLyrics> {
    let tags = tags::read_metadata(Path::new(track_path));
    let resolved = ResolvedLyrics {
        synced_lyrics: sanitize_optional_text(tags.synced_lyrics),
        plain_lyrics: sanitize_optional_text(tags.lyrics),
        lyricist: sanitize_optional_text(tags.lyricist),
    };

    resolved.has_any().then_some(resolved)
}

fn load_cached_lyrics_response(
    conn: &rusqlite::Connection,
    track_id: i64,
) -> Result<Option<LyricsResponse>, String> {
    let cached: Option<(Option<String>, Option<String>, Option<String>)> = conn
        .query_row(
            "SELECT synced_lyrics, plain_lyrics, source FROM lyrics_cache WHERE track_id = ?1",
            rusqlite::params![track_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()
        .map_err(|e| e.to_string())?;

    Ok(cached.and_then(|(synced_lyrics, plain_lyrics, source)| {
        let source_value = source.unwrap_or_else(|| "cache".to_string());
        let resolved = ResolvedLyrics {
            synced_lyrics: sanitize_optional_text(synced_lyrics),
            plain_lyrics: sanitize_optional_text(plain_lyrics),
            lyricist: None,
        };

        if resolved.has_any() {
            return Some(resolved.into_response(track_id, &source_value));
        }

        (source_value == "manual-none").then_some(LyricsResponse {
            track_id,
            synced_lyrics: None,
            plain_lyrics: None,
            lyricist: None,
            source: source_value,
        })
    }))
}

fn cache_lyrics(
    conn: &rusqlite::Connection,
    track_id: i64,
    lyrics: &ResolvedLyrics,
) -> Result<(), String> {
    conn.execute(
        "INSERT OR REPLACE INTO lyrics_cache (track_id, synced_lyrics, plain_lyrics, source) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![
            track_id,
            lyrics.synced_lyrics.as_deref(),
            lyrics.plain_lyrics.as_deref(),
            "lrclib"
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

fn fetch_lrclib_lyrics(track: &TrackLookup) -> Result<Option<ResolvedLyrics>, String> {
    let client = build_http_client()?;

    if let Some(candidate) = fetch_lrclib_get(&client, track)?
        && let Some(lyrics) = lyrics_from_candidate(candidate)
    {
        return Ok(Some(lyrics));
    }

    let Some(search_results) = fetch_lrclib_search(&client, track)? else {
        return Ok(None);
    };

    Ok(select_best_search_candidate(search_results, track))
}

fn build_http_client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .connect_timeout(Duration::from_secs(3))
        .timeout(Duration::from_secs(10))
        .user_agent("Sermon/0.0.0")
        .build()
        .map_err(|e| e.to_string())
}

fn fetch_lrclib_get(
    client: &reqwest::blocking::Client,
    track: &TrackLookup,
) -> Result<Option<LrclibCandidate>, String> {
    let duration_secs = track.duration_secs.unwrap_or(0);
    let url = format!(
        "{}/api/get?artist_name={}&track_name={}&album_name={}&duration={}",
        LRCLIB_BASE_URL,
        urlencoding::encode(&track.artist),
        urlencoding::encode(&track.title),
        urlencoding::encode(&track.album),
        duration_secs
    );

    let Some(response) = perform_get_with_retry(client, &url)? else {
        return Ok(None);
    };

    match response.json::<LrclibCandidate>() {
        Ok(candidate) => Ok(Some(candidate)),
        Err(error) => {
            warn!(error = %error, "lrclib_get_parse_failed");
            Ok(None)
        }
    }
}

fn fetch_lrclib_search(
    client: &reqwest::blocking::Client,
    track: &TrackLookup,
) -> Result<Option<Vec<LrclibCandidate>>, String> {
    let query = format!("{} {}", track.artist, track.title)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    if query.is_empty() {
        return Ok(None);
    }

    let url = format!(
        "{}/api/search?q={}",
        LRCLIB_BASE_URL,
        urlencoding::encode(&query)
    );

    let Some(response) = perform_get_with_retry(client, &url)? else {
        return Ok(None);
    };

    match response.json::<Vec<LrclibCandidate>>() {
        Ok(items) => Ok(Some(items)),
        Err(error) => {
            warn!(error = %error, "lrclib_search_parse_failed");
            Ok(None)
        }
    }
}

fn perform_get_with_retry(
    client: &reqwest::blocking::Client,
    url: &str,
) -> Result<Option<reqwest::blocking::Response>, String> {
    for attempt in 0..=1 {
        match client.get(url).send() {
            Ok(response) => {
                if response.status().is_success() {
                    return Ok(Some(response));
                }

                if response.status().is_server_error() && attempt == 0 {
                    thread::sleep(Duration::from_millis(RETRY_BACKOFF_MS));
                    continue;
                }

                return Ok(None);
            }
            Err(error) => {
                if is_transient_request_error(&error) && attempt == 0 {
                    thread::sleep(Duration::from_millis(RETRY_BACKOFF_MS));
                    continue;
                }

                return Ok(None);
            }
        }
    }

    Ok(None)
}

fn is_transient_request_error(error: &reqwest::Error) -> bool {
    error.is_connect() || error.is_timeout()
}

fn select_best_search_candidate(
    candidates: Vec<LrclibCandidate>,
    track: &TrackLookup,
) -> Option<ResolvedLyrics> {
    let mut best: Option<(i32, ResolvedLyrics)> = None;

    for candidate in candidates {
        let score = score_candidate(&candidate, track);

        let Some(lyrics) = lyrics_from_candidate(candidate) else {
            continue;
        };

        if best
            .as_ref()
            .map(|(best_score, _)| score > *best_score)
            .unwrap_or(true)
        {
            best = Some((score, lyrics));
        }
    }

    best.map(|(_, lyrics)| lyrics)
}

fn lyrics_from_candidate(candidate: LrclibCandidate) -> Option<ResolvedLyrics> {
    let resolved = ResolvedLyrics {
        synced_lyrics: sanitize_optional_text(candidate.synced_lyrics),
        plain_lyrics: sanitize_optional_text(candidate.plain_lyrics),
        lyricist: None,
    };

    resolved.has_any().then_some(resolved)
}

fn score_candidate(candidate: &LrclibCandidate, track: &TrackLookup) -> i32 {
    let mut score = 0;

    let normalized_track_title = normalize_string(&track.title);
    let normalized_track_artist = normalize_string(&track.artist);
    let normalized_candidate_title =
        normalize_string(candidate.track_name.as_deref().unwrap_or(""));
    let normalized_candidate_artist =
        normalize_string(candidate.artist_name.as_deref().unwrap_or(""));

    if !normalized_track_title.is_empty() && normalized_candidate_title == normalized_track_title {
        score += SCORE_EXACT_TITLE;
    }

    if !normalized_track_artist.is_empty() && normalized_candidate_artist == normalized_track_artist
    {
        score += SCORE_EXACT_ARTIST;
    }

    if let (Some(track_duration), Some(candidate_duration)) = (
        track.duration_secs,
        duration_secs_from_candidate(candidate.duration),
    ) && (track_duration - candidate_duration).abs() <= DURATION_TOLERANCE_SECS
    {
        score += SCORE_DURATION_TOLERANCE;
    }

    if sanitize_optional_text(candidate.synced_lyrics.clone()).is_some() {
        score += SCORE_SYNCED_BONUS;
    }

    score
}

fn normalize_string(value: &str) -> String {
    value
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn sanitize_optional_text(value: Option<String>) -> Option<String> {
    value.and_then(|text| {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn duration_ms_to_secs(duration_ms: Option<i64>) -> Option<i64> {
    duration_ms.map(|ms| (ms.max(0) + 500) / 1_000)
}

fn duration_secs_from_candidate(duration: Option<f64>) -> Option<i64> {
    duration.map(|seconds| seconds.round() as i64)
}

fn resolve_precedence(
    track_id: i64,
    embedded: Option<ResolvedLyrics>,
    cached: Option<ResolvedLyrics>,
    lrclib: Option<ResolvedLyrics>,
) -> LyricsResponse {
    if let Some(lyrics) = embedded {
        return lyrics.into_response(track_id, "embedded");
    }

    if let Some(lyrics) = cached {
        return lyrics.into_response(track_id, "cache");
    }

    if let Some(lyrics) = lrclib {
        return lyrics.into_response(track_id, "lrclib");
    }

    LyricsResponse {
        track_id,
        synced_lyrics: None,
        plain_lyrics: None,
        lyricist: None,
        source: "none".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_track(title: &str, artist: &str, duration_secs: Option<i64>) -> TrackLookup {
        TrackLookup {
            path: String::new(),
            title: title.to_string(),
            artist: artist.to_string(),
            album: String::new(),
            duration_secs,
        }
    }

    #[test]
    fn test_normalize_string() {
        assert_eq!(
            normalize_string("  HeLLo   WORLD\n\tagain  "),
            "hello world again"
        );
    }

    #[test]
    fn test_score_candidate_exact_match() {
        let track = test_track("Song Title", "Artist Name", Some(180));
        let candidate = LrclibCandidate {
            track_name: Some("Song Title".to_string()),
            artist_name: Some("Artist Name".to_string()),
            duration: Some(181.0),
            synced_lyrics: Some("[00:01.00]line".to_string()),
            plain_lyrics: None,
        };

        assert_eq!(score_candidate(&candidate, &track), 28);
    }

    #[test]
    fn test_score_candidate_partial() {
        let track = test_track("Song Title", "Artist Name", Some(180));
        let candidate = LrclibCandidate {
            track_name: Some("Song Title (Live)".to_string()),
            artist_name: Some("Artist Name".to_string()),
            duration: Some(230.0),
            synced_lyrics: Some("[00:01.00]line".to_string()),
            plain_lyrics: None,
        };

        assert_eq!(score_candidate(&candidate, &track), 13);
    }

    #[test]
    fn test_score_candidate_duration_tolerance() {
        let track = test_track("Song Title", "Artist Name", Some(180));

        let within_tolerance = LrclibCandidate {
            track_name: Some("Song Title".to_string()),
            artist_name: Some("Artist Name".to_string()),
            duration: Some(182.0),
            synced_lyrics: None,
            plain_lyrics: Some("plain".to_string()),
        };

        let outside_tolerance = LrclibCandidate {
            track_name: Some("Song Title".to_string()),
            artist_name: Some("Artist Name".to_string()),
            duration: Some(190.0),
            synced_lyrics: None,
            plain_lyrics: Some("plain".to_string()),
        };

        let near_score = score_candidate(&within_tolerance, &track);
        let far_score = score_candidate(&outside_tolerance, &track);

        assert_eq!(near_score, far_score + SCORE_DURATION_TOLERANCE);
    }

    #[test]
    fn test_score_candidate_all_zero_when_no_signals_match() {
        let track = test_track("Song Title", "Artist Name", Some(180));
        let candidate = LrclibCandidate {
            track_name: Some("Different Title".to_string()),
            artist_name: Some("Different Artist".to_string()),
            duration: Some(999.0),
            synced_lyrics: None,
            plain_lyrics: Some("plain".to_string()),
        };

        let score = score_candidate(&candidate, &track);
        assert_eq!(
            score, 0,
            "non-matching title/artist/duration with no synced lyrics should score zero"
        );
    }

    #[test]
    fn test_score_candidate_without_track_duration_skips_duration_bonus() {
        let track = test_track("Song Title", "Artist Name", None);
        let candidate = LrclibCandidate {
            track_name: Some("Song Title".to_string()),
            artist_name: Some("Artist Name".to_string()),
            duration: Some(182.0),
            synced_lyrics: None,
            plain_lyrics: Some("plain".to_string()),
        };

        let score = score_candidate(&candidate, &track);
        assert_eq!(
            score,
            SCORE_EXACT_TITLE + SCORE_EXACT_ARTIST,
            "duration bonus must not apply when track duration is unavailable"
        );
    }

    #[test]
    fn test_select_best_search_candidate_tie_keeps_first_match() {
        let track = test_track("Song Title", "Artist Name", Some(180));
        let candidates = vec![
            LrclibCandidate {
                track_name: Some("Song Title".to_string()),
                artist_name: Some("Artist Name".to_string()),
                duration: Some(181.0),
                synced_lyrics: None,
                plain_lyrics: Some("first candidate lyrics".to_string()),
            },
            LrclibCandidate {
                track_name: Some("Song Title".to_string()),
                artist_name: Some("Artist Name".to_string()),
                duration: Some(181.0),
                synced_lyrics: None,
                plain_lyrics: Some("second candidate lyrics".to_string()),
            },
        ];

        let best = select_best_search_candidate(candidates, &track)
            .expect("a tied candidate with lyrics should still resolve");
        assert_eq!(
            best.plain_lyrics.as_deref(),
            Some("first candidate lyrics"),
            "tie-breaking should preserve the first highest-scoring lyrics candidate"
        );
    }

    #[test]
    fn test_select_best_search_candidate_skips_candidates_without_lyrics() {
        let track = test_track("Song Title", "Artist Name", Some(180));
        let candidates = vec![
            LrclibCandidate {
                track_name: Some("Song Title".to_string()),
                artist_name: Some("Artist Name".to_string()),
                duration: Some(180.0),
                synced_lyrics: None,
                plain_lyrics: None,
            },
            LrclibCandidate {
                track_name: Some("Different Title".to_string()),
                artist_name: Some("Different Artist".to_string()),
                duration: Some(240.0),
                synced_lyrics: None,
                plain_lyrics: Some("fallback lyrics".to_string()),
            },
        ];

        let best = select_best_search_candidate(candidates, &track)
            .expect("a lower-scoring candidate with lyrics should be chosen over empty entries");
        assert_eq!(
            best.plain_lyrics.as_deref(),
            Some("fallback lyrics"),
            "candidates with no synced/plain lyrics must be excluded from selection"
        );
    }

    #[test]
    fn test_select_best_search_candidate_returns_none_when_all_lyrics_empty() {
        let track = test_track("Song Title", "Artist Name", Some(180));
        let candidates = vec![
            LrclibCandidate {
                track_name: Some("Song Title".to_string()),
                artist_name: Some("Artist Name".to_string()),
                duration: Some(180.0),
                synced_lyrics: None,
                plain_lyrics: None,
            },
            LrclibCandidate {
                track_name: Some("Song Title".to_string()),
                artist_name: Some("Artist Name".to_string()),
                duration: Some(180.0),
                synced_lyrics: Some("   ".to_string()),
                plain_lyrics: Some("\n\t".to_string()),
            },
        ];

        let best = select_best_search_candidate(candidates, &track);
        assert!(
            best.is_none(),
            "search should return None when every candidate has empty/whitespace-only lyrics"
        );
    }

    #[test]
    fn test_precedence_embedded_first() {
        let embedded = ResolvedLyrics {
            synced_lyrics: Some("[00:01.00]embedded".to_string()),
            plain_lyrics: None,
            lyricist: None,
        };
        let cached = ResolvedLyrics {
            synced_lyrics: None,
            plain_lyrics: Some("cached".to_string()),
            lyricist: None,
        };

        let response = resolve_precedence(7, Some(embedded), Some(cached), None);

        assert_eq!(response.source, "embedded");
        assert_eq!(
            response.synced_lyrics.as_deref(),
            Some("[00:01.00]embedded")
        );
        assert!(response.plain_lyrics.is_none());
    }

    #[test]
    fn test_precedence_cache_used_when_embedded_missing() {
        let cached = ResolvedLyrics {
            synced_lyrics: None,
            plain_lyrics: Some("cached body".to_string()),
            lyricist: None,
        };
        let lrclib = ResolvedLyrics {
            synced_lyrics: Some("[00:01.00]lrclib".to_string()),
            plain_lyrics: Some("lrclib body".to_string()),
            lyricist: None,
        };

        let response = resolve_precedence(17, None, Some(cached), Some(lrclib));
        assert_eq!(
            response.source, "cache",
            "cache should win over lrclib when embedded lyrics are absent"
        );
        assert_eq!(
            response.plain_lyrics.as_deref(),
            Some("cached body"),
            "resolved lyrics should come from cache source"
        );
    }

    #[test]
    fn test_precedence_none_when_all_sources_missing() {
        let response = resolve_precedence(23, None, None, None);
        assert_eq!(
            response.source, "none",
            "missing embedded/cache/lrclib lyrics should return explicit none source"
        );
        assert!(
            response.synced_lyrics.is_none() && response.plain_lyrics.is_none(),
            "none source should not carry any lyric payload"
        );
    }
}
