/**
 * Lyrics fixtures for mock mode (SERMON_MOCK=1)
 * Keyed by track id (numeric TrackEventData.id)
 */

export type LyricsFixture = {
  trackId: number;
  lines: string[];
};

/**
 * Mock lyrics data keyed by track ID.
 * In mock mode, track IDs are 0-based fixture indices.
 */
export const lyricsFixtures: Record<number, string[]> = {
  // Track 0 - First fixture track
  0: [
    "In the stillness of the morning light",
    "I find myself lost in thought",
    "The melodies that drift through time",
    "Carry echoes of what we've sought",
    "",
    "Through the valleys and the peaks we climb",
    "Every note becomes a prayer",
    "In the silence between the beats",
    "We discover what's already there",
    "",
    "The rhythm speaks what words cannot",
    "A language only hearts can know",
    "In every rise and gentle fall",
    "We learn to let the music flow",
    "",
    "So let the symphony unfold",
    "Each measure tells a story true",
    "In harmonies both new and old",
    "The music finds its way to you",
  ],
  // Track 1
  1: [
    "Dancing shadows on the wall",
    "Whispered secrets in the hall",
    "Time moves slowly, standing still",
    "Against the winter's bitter chill",
    "",
    "Memories like falling snow",
    "Drifting down from long ago",
    "Catching light before they land",
    "Melting gently in my hand",
  ],
  // Track 2
  2: [
    "City lights begin to glow",
    "Streets alive with evening flow",
    "Strangers passing, stories untold",
    "Dreams of silver, dreams of gold",
    "",
    "In the crowd I search for you",
    "Faces fading from my view",
    "But the music guides me home",
    "Never truly am I alone",
  ],
};

/**
 * Get lyrics for a track by ID.
 * Returns null if no lyrics available.
 */
export function getLyricsForTrack(trackId: number): string[] | null {
  return lyricsFixtures[trackId] ?? null;
}
