import { derived } from 'svelte/store';
import { currentTrack, progress } from './playback';
import { getLyricsForTrack } from '../data/lyricsFixtures';

/**
 * Current lyrics lines for the playing track.
 * Returns null if no lyrics are available.
 */
export const currentLyrics = derived(currentTrack, ($currentTrack) => {
  if (!$currentTrack) return null;
  
  // In mock mode, track ID is the fixture index
  // getLyricsForTrack returns null if not found
  return getLyricsForTrack($currentTrack.id);
});

/**
 * Current active line index based on playback progress.
 * Algorithm: currentLineIndex = clamp(floor(progress * lines.length), 0, lines.length - 1)
 */
export const currentLineIndex = derived(
  [currentLyrics, progress],
  ([$currentLyrics, $progress]) => {
    if (!$currentLyrics || $currentLyrics.length === 0) return 0;
    
    const lineCount = $currentLyrics.length;
    const rawIndex = Math.floor($progress * lineCount);
    
    // Clamp to valid range
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
      totalLines: $currentLyrics.length 
    };
  }
);
