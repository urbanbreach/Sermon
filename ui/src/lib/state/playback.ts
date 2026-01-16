import { writable, derived, get } from 'svelte/store';
import { listen } from '@tauri-apps/api/event';
import type {
  PlaybackStateEvent, NowPlayingEvent, PlaybackPositionEvent,
  QueueChangedEvent, DeviceChangedEvent, AudioDebugEvent, PlaybackErrorEvent,
  TrackEventData, QueueItemData
} from '../types/playback';
import * as api from '../api/playback';
import type { AudioOutputSettings } from '../api/playback';

// Core state
export const playbackState = writable<'playing' | 'paused' | 'stopped'>('stopped');
export const playId = writable<string | null>(null);
export const currentTrack = writable<TrackEventData | null>(null);
export const positionMs = writable<number>(0);
export const playedMs = writable<number>(0);
export const durationMs = writable<number>(0);
export const volume = writable<number>(1.0);

// Queue state
export const queue = writable<QueueItemData[]>([]);
export const currentIndex = writable<number | null>(null);

// Device state
export const currentDevice = writable<{ id: string; name: string; isDefault: boolean } | null>(null);
export const devices = writable<api.AudioDeviceInfo[]>([]);
export const outputSettings = writable<AudioOutputSettings | null>(null);

// Audio debug
export const audioDebug = writable<AudioDebugEvent | null>(null);

// Error state
export const playbackError = writable<PlaybackErrorEvent | null>(null);

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

export async function addToQueue(trackId: number) {
  try {
    await api.queueAdd(trackId);
  } catch (e) {
    console.error('Add to queue failed:', e);
  }
}

export async function togglePlayPause() {
  const state = get(playbackState);
  if (state === 'playing') {
    await api.playbackPause();
  } else if (state === 'paused') {
    await api.playbackResume();
  } else if (state === 'stopped') {
    // If stopped, we might want to restart the current track if available, or do nothing.
    // Ideally the UI should handle "Play" on a specific track if stopped.
    // But if we have a queue, we might be able to resume? 
    // For now let's just log or no-op if stopped, as usually this button becomes "Play" and needs a target or resume signal.
    // Actually, if we are stopped but have a current track/queue, maybe we can just resume?
    // Let's assume resume works if we have context.
     await api.playbackResume();
  }
}

export async function stop() {
  await api.playbackStop();
}

export async function seek(ms: number) {
  await api.playbackSeek(ms);
}

export async function next() {
  await api.playbackNext();
}

export async function previous() {
  await api.playbackPrevious();
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
  listen<PlaybackStateEvent>('evt_playback_state', (event) => {
    playbackState.set(event.payload.state as 'playing' | 'paused' | 'stopped');
    playId.set(event.payload.play_id);
  });

  listen<NowPlayingEvent>('evt_now_playing', (event) => {
    currentTrack.set(event.payload.track);
    positionMs.set(event.payload.position_ms);
    durationMs.set(event.payload.track.duration_ms || 0);
  });

  listen<PlaybackPositionEvent>('evt_playback_position', (event) => {
    positionMs.set(event.payload.position_ms);
    playedMs.set(event.payload.played_ms);
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

  listen<PlaybackErrorEvent>('evt_playback_error', (event) => {
    playbackError.set(event.payload);
  });

  // Load initial volume
  api.getVolume().then(v => volume.set(v)).catch(() => {});
  
  // Load settings
  loadOutputSettings();
}
