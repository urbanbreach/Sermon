import { derived, writable } from 'svelte/store';
import { currentTrack, positionMs, progress } from './playback';
import { getLyricsForTrack as getFixtureLyricsForTrack } from '../data/lyricsFixtures';
import { getLyricsForTrack as getLyricsForTrackApi } from '../api/lyrics';
import type {
  LyricsResponse,
  LyricsStatus,
  ResolvedLyrics,
  TimedLine,
} from '../types/lyrics';

const OFFSET_TAG_PATTERN = /\[offset:([+-]?\d+)\]/i;
const TIMESTAMP_PATTERN = /\[(\d{1,2}):(\d{2})(?:\.(\d{2,3}))?\]/g;
const METADATA_TAG_PATTERN = /\[(ar|ti|al|length):[^\]]*\]/gi;

let requestToken = 0;
let inFlightTrackId: number | null = null;
let inFlightPromise: Promise<void> | null = null;
let inFlightMarker: symbol | null = null;
let observedTrackId: number | null = null;

export const lyricsStatus = writable<LyricsStatus>('idle');
export const lyricsData = writable<ResolvedLyrics | null>(null);

function splitPlainLyrics(plainLyrics: string | null): string[] {
  if (!plainLyrics || plainLyrics.trim().length === 0) {
    return [];
  }

  const normalized = plainLyrics.replace(/\r\n/g, '\n').replace(/\r/g, '\n');
  return normalized.split('\n');
}

function toTimedLineMs(minutesRaw: string, secondsRaw: string, fractionRaw?: string): number {
  const minutes = Number.parseInt(minutesRaw, 10);
  const seconds = Number.parseInt(secondsRaw, 10);

  let fractionMs = 0;
  if (fractionRaw) {
    fractionMs = Number.parseInt(fractionRaw.padEnd(3, '0'), 10);
  }

  return minutes * 60_000 + seconds * 1_000 + fractionMs;
}

/** Parse an LRC string into sorted timed lines. */
export function parseLrc(lrc: string): TimedLine[] {
  if (!lrc || lrc.trim().length === 0) {
    return [];
  }

  let offsetMs = 0;
  const parsed: TimedLine[] = [];

  for (const rawLine of lrc.split(/\r?\n/)) {
    const line = rawLine.trim();
    if (!line) {
      continue;
    }

    const offsetMatch = line.match(OFFSET_TAG_PATTERN);
    if (offsetMatch) {
      offsetMs = Number.parseInt(offsetMatch[1], 10);
    }

    TIMESTAMP_PATTERN.lastIndex = 0;
    const timestampMatches = Array.from(line.matchAll(TIMESTAMP_PATTERN));
    if (timestampMatches.length === 0) {
      continue;
    }

    const lineWithoutMetadata = line.replace(METADATA_TAG_PATTERN, '');
    const text = lineWithoutMetadata.replace(TIMESTAMP_PATTERN, '').trim();

    for (const match of timestampMatches) {
      const [, minutesRaw, secondsRaw, fractionRaw] = match;
      parsed.push({
        timeMs: toTimedLineMs(minutesRaw, secondsRaw, fractionRaw),
        text,
      });
    }
  }

  if (parsed.length === 0) {
    return [];
  }

  const withOffset = parsed.map((line) => ({
    ...line,
    timeMs: line.timeMs + offsetMs,
  }));

  withOffset.sort((a, b) => a.timeMs - b.timeMs);
  return withOffset;
}

/**
 * Find the last line index where line.timeMs <= positionMs.
 * Returns 0 if no line has started yet or if lines are empty.
 */
export function findActiveLineIndex(lines: TimedLine[], positionMsValue: number): number {
  if (lines.length === 0) {
    return 0;
  }

  let low = 0;
  let high = lines.length - 1;

  while (low <= high) {
    const mid = Math.floor((low + high) / 2);
    if (lines[mid].timeMs <= positionMsValue) {
      low = mid + 1;
    } else {
      high = mid - 1;
    }
  }

  return Math.max(0, Math.min(high, lines.length - 1));
}

function toResolvedLyrics(response: LyricsResponse): ResolvedLyrics | null {
  if (response.source === 'none') {
    return null;
  }

  const lines = response.syncedLyrics ? parseLrc(response.syncedLyrics) : [];
  const plainLines = splitPlainLyrics(response.plainLyrics);

  if (lines.length === 0 && plainLines.length === 0) {
    return null;
  }

  return {
    trackId: response.trackId,
    lines,
    plainLines,
    isSynced: lines.length > 0,
    source: response.source,
  };
}

function resolveMockLyrics(trackId: number): ResolvedLyrics | null {
  const fixtureLines = getFixtureLyricsForTrack(trackId);
  if (!fixtureLines || fixtureLines.length === 0) {
    return null;
  }

  return {
    trackId,
    lines: [],
    plainLines: fixtureLines,
    isSynced: false,
    source: 'embedded',
  };
}

function resetLyricsState() {
  requestToken += 1;
  inFlightTrackId = null;
  inFlightPromise = null;
  inFlightMarker = null;
  lyricsData.set(null);
  lyricsStatus.set('idle');
}

export async function fetchLyricsForTrack(trackId: number): Promise<void> {
  if (inFlightTrackId === trackId && inFlightPromise) {
    return inFlightPromise;
  }

  const token = ++requestToken;
  const marker = Symbol(`lyrics-${trackId}-${token}`);

  lyricsStatus.set('loading');

  const request = (async () => {
    try {
      const resolved =
        import.meta.env.SERMON_MOCK === '1'
          ? resolveMockLyrics(trackId)
          : toResolvedLyrics(await getLyricsForTrackApi(trackId));

      if (token !== requestToken) {
        return;
      }

      if (!resolved) {
        lyricsData.set(null);
        lyricsStatus.set('not_found');
        return;
      }

      lyricsData.set(resolved);
      lyricsStatus.set('ready');
    } catch (error) {
      if (token !== requestToken) {
        return;
      }

      lyricsData.set(null);
      lyricsStatus.set('error');
      console.error('Failed to fetch lyrics for track', trackId, error);
    } finally {
      if (inFlightMarker === marker) {
        inFlightTrackId = null;
        inFlightPromise = null;
        inFlightMarker = null;
      }
    }
  })();

  inFlightTrackId = trackId;
  inFlightPromise = request;
  inFlightMarker = marker;

  return request;
}

currentTrack.subscribe((track) => {
  if (!track) {
    observedTrackId = null;
    resetLyricsState();
    return;
  }

  if (track.id === observedTrackId) {
    return;
  }

  observedTrackId = track.id;
  void fetchLyricsForTrack(track.id);
});

/**
 * Current lyrics lines for the playing track.
 * Returns null if no lyrics are available.
 */
export const currentLyrics = derived(lyricsData, ($lyricsData): string[] | null => {
  if (!$lyricsData) {
    return null;
  }

  if ($lyricsData.isSynced) {
    return $lyricsData.lines.map((line) => line.text);
  }

  return $lyricsData.plainLines.length > 0 ? $lyricsData.plainLines : null;
});

/**
 * Current active line index.
 * - Synced lyrics: timestamp-based binary search using playback positionMs
 * - Unsynced/plain lyrics fallback: progress-based index
 */
export const currentLineIndex = derived(
  [lyricsData, positionMs, progress],
  ([$lyricsData, $positionMs, $progress]) => {
    if (!$lyricsData) {
      return 0;
    }

    if ($lyricsData.isSynced && $lyricsData.lines.length > 0) {
      return findActiveLineIndex($lyricsData.lines, $positionMs);
    }

    const lineCount = $lyricsData.plainLines.length;
    if (lineCount === 0) {
      return 0;
    }

    const rawIndex = Math.floor($progress * lineCount);
    return Math.max(0, Math.min(rawIndex, lineCount - 1));
  }
);

/**
 * Context window: 2 lines before, active line, 2 lines after.
 * Returns { lines, activeIndex } where activeIndex is relative to the returned lines array.
 */
export const lyricsContext = derived(
  [currentLyrics, currentLineIndex],
  ([$currentLyrics, $currentLineIndex]) => {
    if (!$currentLyrics || $currentLyrics.length === 0) {
      return { lines: [], activeIndex: 0, totalLines: 0 };
    }

    const contextBefore = 2;
    const contextAfter = 2;

    const startIndex = Math.max(0, $currentLineIndex - contextBefore);
    const endIndex = Math.min($currentLyrics.length, $currentLineIndex + contextAfter + 1);

    const lines = $currentLyrics.slice(startIndex, endIndex);
    const activeIndex = $currentLineIndex - startIndex;

    return {
      lines,
      activeIndex,
      totalLines: $currentLyrics.length,
    };
  }
);
