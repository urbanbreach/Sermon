import { writable, derived, get } from 'svelte/store';
import { listen } from '@tauri-apps/api/event';
import { getTrackById } from '../api/library';
import { loadAlbumTracksIfNeeded } from './rightRail';
import type { TrackRow } from '../types/library';
import type {
  PlaybackStateEvent, NowPlayingEvent, PlaybackPositionEvent,
  QueueChangedEvent, DeviceChangedEvent, AudioDebugEvent, PlaybackErrorEvent,
  TrackEventData, QueueItemData, TrackMarkedMissingEvent
} from '../types/playback';
import type { AudioTelemetryEvent } from '../types/telemetry';
import * as api from '../api/playback';
import type { AudioOutputSettings, AsioDriverInfo } from '../api/playback';
import { Fixtures } from '../data/fixtures';

// Core state
export const playbackState = writable<'playing' | 'paused' | 'stopped'>('stopped');
export const currentTrack = writable<TrackEventData | null>(null);
export const positionMs = writable<number>(0);
export const durationMs = writable<number>(0);
export const volume = writable<number>(1.0);

// Queue state
export const queue = writable<QueueItemData[]>([]);
export const currentIndex = writable<number | null>(null);

// Device state
export const currentDevice = writable<{ id: string; name: string; isDefault: boolean } | null>(null);
export const devices = writable<api.AudioDeviceInfo[]>([]);
export const outputSettings = writable<AudioOutputSettings | null>(null);
export const asioDrivers = writable<AsioDriverInfo[]>([]);

// Audio debug
export const audioDebug = writable<AudioDebugEvent | null>(null);

// Audio telemetry (1 Hz diagnostics snapshot)
export const audioTelemetry = writable<AudioTelemetryEvent | null>(null);

// Error state
export const playbackError = writable<PlaybackErrorEvent | null>(null);

// Full track data (includes year, genre, disc_no, track_no, etc.)
export const currentTrackFull = writable<TrackRow | null>(null);


// Derived
export const isPlaying = derived(playbackState, $s => $s === 'playing');
export const progress = derived([positionMs, durationMs], ([$pos, $dur]) => 
  $dur > 0 ? $pos / $dur : 0
);

// Actions
export async function playNow(trackId: number) {
  try {
    await api.queuePlayNow(trackId);
  } catch (e) {
    console.error('Play failed:', e);
  }
}

export async function playNowWithQueue(trackIds: number[], startIndex: number) {
  try {
    await api.queueSetAndPlay(trackIds, startIndex);
  } catch (e) {
    console.error('Play with queue failed:', e);
  }
}

export async function addToQueue(trackId: number) {
  try {
    await api.queueAdd(trackId);
  } catch (e) {
    console.error('Add to queue failed:', e);
  }
}

export async function addToQueueNext(trackIds: number[]) {
  try {
    await api.queueAddNext(trackIds);
  } catch (e) {
    console.error('Add to queue next failed:', e);
  }
}

export async function togglePlayPause() {
  const state = get(playbackState);
  console.log('[Playback] togglePlayPause called, current state:', state);
  try {
    if (state === 'playing') {
      await api.playbackPause();
    } else if (state === 'paused') {
      await api.playbackResume();
    } else if (state === 'stopped') {
      // If stopped, try to resume - this works if there's a track in the queue
      await api.playbackResume();
    }
  } catch (e) {
    console.error('[Playback] togglePlayPause failed:', e);
  }
}

export async function restorePlaybackSession(): Promise<void> {
  await api.restorePlaybackSession();
}

export async function stop() {
  await api.playbackStop();
}

export async function seek(ms: number) {
  await api.playbackSeek(ms);
}

export async function next() {
  console.log('[Playback] next called');
  try {
    await api.playbackNext();
  } catch (e) {
    console.error('[Playback] next failed:', e);
  }
}

export async function previous() {
  console.log('[Playback] previous called');
  try {
    await api.playbackPrevious();
  } catch (e) {
    console.error('[Playback] previous failed:', e);
  }
}

export async function setVolume(vol: number) {
  volume.set(vol);
  await api.setVolume(vol);
}

export async function loadDevices() {
  try {
    const list = await api.listDevices();
    devices.set(list);
  } catch (e) {
    console.error('Failed to load devices:', e);
  }
}

export async function loadAsioDrivers() {
  try {
    const list = await api.listAsioDrivers();
    asioDrivers.set(list);
  } catch (e) {
    console.error('Failed to load ASIO drivers:', e);
  }
}

export async function selectDevice(deviceId: string) {
  await api.setDevice(deviceId);
}

export async function switchToDefault() {
  await api.setDevice('default');
  playbackError.set(null);
}

export async function loadOutputSettings() {
  try {
    const settings = await api.getOutputSettings();
    outputSettings.set(settings);
  } catch (e) {
    console.error('Failed to load output settings:', e);
  }
}

export async function saveOutputSettings(settings: AudioOutputSettings) {
  try {
    await api.setOutputSettings(settings);
    outputSettings.set(settings);
  } catch (e) {
    console.error('Failed to save output settings:', e);
  }
}

// Event listeners
export function initPlaybackListeners(): void {
  // Snapshot mode: seed stores deterministically and return immediately
  if (import.meta.env.SERMON_MOCK === '1' && import.meta.env.SERMON_SNAPSHOT === '1') {
    const fixtureTrack = Fixtures.getTracks()[0];
    const artist = Fixtures.getArtist(fixtureTrack.artistId);
    const album = Fixtures.getAlbum(fixtureTrack.albumId);
    
    currentTrack.set({
      id: 0, // 0-based fixture index
      title: fixtureTrack.title,
      artist: artist?.name ?? '',
      album: album?.title ?? '',
      duration_ms: fixtureTrack.durationMs,
    });
    playbackState.set('playing');
    durationMs.set(fixtureTrack.durationMs);
    positionMs.set(0);
    queue.set([]);
    currentIndex.set(null);
    
    return; // No listeners, no API calls
  }

  listen<PlaybackStateEvent>('evt_playback_state', (event) => {
    playbackState.set(event.payload.state as 'playing' | 'paused' | 'stopped');
  });

  listen<NowPlayingEvent>('evt_now_playing', (event) => {
    currentTrack.set(event.payload.track);
    positionMs.set(event.payload.position_ms);
    durationMs.set(event.payload.track.duration_ms || 0);

    // Fetch full track metadata
    if (event.payload.track.id) {
      getTrackById(event.payload.track.id).then(fullTrack => {
        currentTrackFull.set(fullTrack);
        loadAlbumTracksIfNeeded();
      }).catch(err => {
        console.warn('Failed to fetch full track data:', err);
        currentTrackFull.set(null);
      });
    }
  });

  listen<PlaybackPositionEvent>('evt_playback_position', (event) => {
    positionMs.set(event.payload.position_ms);
    durationMs.set(event.payload.duration_ms);
  });

  listen<QueueChangedEvent>('evt_queue_changed', (event) => {
    queue.set(event.payload.queue);
    currentIndex.set(event.payload.current_index);
  });

  listen<DeviceChangedEvent>('evt_device_changed', (event) => {
    currentDevice.set({
      id: event.payload.device_id,
      name: event.payload.device_name,
      isDefault: event.payload.is_default
    });
  });

  listen<AudioDebugEvent>('evt_audio_debug', (event) => {
    audioDebug.set(event.payload);
  });

  listen<AudioTelemetryEvent>('evt_audio_telemetry', (event) => {
    audioTelemetry.set(event.payload);
  });

  listen<PlaybackErrorEvent>('evt_playback_error', (event) => {
    playbackError.set(event.payload);
  });

  // Listen for tracks marked as missing - triggers library refresh
  listen<TrackMarkedMissingEvent>('evt_track_marked_missing', (event) => {
    console.warn(`Track marked missing: ${event.payload.path} (ID: ${event.payload.track_id})`);
    // Dispatch custom event that views can listen to for refresh
    window.dispatchEvent(new CustomEvent('sermon:track-marked-missing', { 
      detail: event.payload 
    }));
  });


  // Load initial volume
  api.getVolume().then(v => volume.set(v)).catch(() => {});
  
  // Load settings
  loadOutputSettings();
}
