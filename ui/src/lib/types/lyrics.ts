/** Status of lyrics resolution */
export type LyricsStatus = 'idle' | 'loading' | 'ready' | 'not_found' | 'error';

/** A single timed lyric line */
export interface TimedLine {
  timeMs: number; // timestamp in milliseconds
  text: string; // lyric text
}

/** Response from cmd_lyrics_get_for_track */
export interface LyricsResponse {
  trackId: number;
  syncedLyrics: string | null; // LRC format
  plainLyrics: string | null; // plain text
  source: string; // "embedded" | "cache" | "lrclib" | "none"
}

/** Resolved lyrics data */
export interface ResolvedLyrics {
  trackId: number;
  lines: TimedLine[]; // empty if plain-only
  plainLines: string[]; // plain text fallback lines
  isSynced: boolean; // true if LRC timestamps exist
  source: string;
}
