import { get } from 'svelte/store';
import { beforeEach, describe, expect, test, vi } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(),
}));

vi.mock('../api/library', () => ({
  getTrackById: vi.fn(),
}));

vi.mock('./rightRail', () => ({
  loadAlbumTracksIfNeeded: vi.fn(),
}));

import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getTrackById } from '../api/library';
import { loadAlbumTracksIfNeeded } from './rightRail';
import {
  audioDebug,
  currentDevice,
  currentIndex,
  currentTrack,
  currentTrackFull,
  durationMs,
  initPlaybackListeners,
  outputSettings,
  playbackError,
  playbackState,
  positionMs,
  queue,
  seek,
  togglePlayPause,
  volume,
} from './playback';

type ListenerEvent = { payload: unknown };
type ListenerCallback = (event: ListenerEvent) => void;

async function flushMicrotasks(passes = 4): Promise<void> {
  for (let i = 0; i < passes; i += 1) {
    await Promise.resolve();
  }
}

function getListenerCallback(eventName: string): ListenerCallback {
  const listenMock = vi.mocked(listen);
  const entry = listenMock.mock.calls.find(([name]) => name === eventName);
  if (!entry) {
    throw new Error(`Listener not registered: ${eventName}`);
  }

  return entry[1] as ListenerCallback;
}

describe('playback store actions and listeners', () => {
  const invokeMock = vi.mocked(invoke);
  const listenMock = vi.mocked(listen);
  const getTrackByIdMock = vi.mocked(getTrackById);
  const loadAlbumTracksIfNeededMock = vi.mocked(loadAlbumTracksIfNeeded);

  beforeEach(() => {
    invokeMock.mockReset();
    listenMock.mockReset();
    getTrackByIdMock.mockReset();
    loadAlbumTracksIfNeededMock.mockReset();

    invokeMock.mockResolvedValue(undefined);

    playbackState.set('stopped');
    currentTrack.set(null);
    currentTrackFull.set(null);
    positionMs.set(0);
    durationMs.set(0);
    queue.set([]);
    currentIndex.set(null);
    currentDevice.set(null);
    audioDebug.set(null);
    playbackError.set(null);
    volume.set(1);
    outputSettings.set(null);
  });

  test('togglePlayPause: dispatches pause command while playing', async () => {
    playbackState.set('playing');

    await togglePlayPause();

    expect(invokeMock).toHaveBeenCalledWith('cmd_playback_pause');
  });

  test('togglePlayPause: dispatches resume command while paused/stopped', async () => {
    playbackState.set('paused');
    await togglePlayPause();
    expect(invokeMock).toHaveBeenLastCalledWith('cmd_playback_resume');

    playbackState.set('stopped');
    await togglePlayPause();
    expect(invokeMock).toHaveBeenLastCalledWith('cmd_playback_resume');
  });

  test('seek: dispatches seek command with floored position', async () => {
    await seek(12_345.98);

    expect(invokeMock).toHaveBeenCalledWith('cmd_playback_seek', { positionMs: 12_345 });
  });

  test('togglePlayPause: rejected invoke does not crash store action', async () => {
    playbackState.set('playing');
    invokeMock.mockRejectedValueOnce(new Error('pause failed'));

    await expect(togglePlayPause()).resolves.toBeUndefined();
    expect(get(playbackState)).toBe('playing');
  });

  test('initPlaybackListeners: event callbacks update playback stores', async () => {
    invokeMock.mockImplementation((command: string) => {
      if (command === 'cmd_volume_get') {
        return Promise.resolve(0.35);
      }

      if (command === 'cmd_output_get_settings') {
        return Promise.resolve({
          mode: 'shared',
          policy: 'compatibility',
          fade: true,
          timing: 'event',
        });
      }

      return Promise.resolve(undefined);
    });

    getTrackByIdMock.mockResolvedValue({
      id: 77,
      libraryFolderId: 1,
      path: '/mock/track-77.flac',
      title: 'Track 77',
      album: 'Album 77',
      artist: 'Artist 77',
      isMissing: false,
    });

    initPlaybackListeners();

    expect(listenMock).toHaveBeenCalled();

    getListenerCallback('evt_playback_state')({
      payload: { state: 'playing', play_id: 'play-1', track_id: 77 },
    });

    getListenerCallback('evt_playback_position')({
      payload: {
        play_id: 'play-1',
        position_ms: 5_000,
        played_ms: 5_000,
        duration_ms: 210_000,
      },
    });

    getListenerCallback('evt_queue_changed')({
      payload: {
        play_id: 'play-1',
        current_index: 0,
        queue: [{ track_id: 77, title: 'Track 77' }],
      },
    });

    getListenerCallback('evt_device_changed')({
      payload: {
        device_id: 'default',
        device_name: 'Default Device',
        is_default: true,
      },
    });

    getListenerCallback('evt_audio_debug')({
      payload: {
        output_mode: 'shared',
        policy: 'compatibility',
        conversion: 'none',
        gain_mode: 'unity',
        fade_enabled: true,
        exclusive_active: false,
        bit_perfect: 'no',
        bit_perfect_reason: 'shared mode',
        device_id: 'default',
        device_name: 'Default Device',
        output_format: { sample_rate: 44_100, bit_depth: 16, channels: 2 },
        decode_format: { sample_rate: 44_100, bit_depth: 16, channels: 2 },
      },
    });

    getListenerCallback('evt_playback_error')({
      payload: {
        code: 'PLAYBACK_TEST',
        message: 'recoverable test error',
        recoverable: true,
      },
    });

    getListenerCallback('evt_now_playing')({
      payload: {
        play_id: 'play-1',
        position_ms: 6_000,
        track: {
          id: 77,
          title: 'Track 77',
          artist: 'Artist 77',
          album: 'Album 77',
          duration_ms: 210_000,
        },
      },
    });

    await flushMicrotasks();

    expect(get(playbackState)).toBe('playing');
    expect(get(positionMs)).toBe(6_000);
    expect(get(durationMs)).toBe(210_000);
    expect(get(queue)).toEqual([{ track_id: 77, title: 'Track 77' }]);
    expect(get(currentIndex)).toBe(0);
    expect(get(currentDevice)).toEqual({ id: 'default', name: 'Default Device', isDefault: true });
    expect(get(audioDebug)?.device_name).toBe('Default Device');
    expect(get(playbackError)?.code).toBe('PLAYBACK_TEST');
    expect(get(currentTrack)?.id).toBe(77);
    expect(get(currentTrackFull)?.id).toBe(77);
    expect(get(volume)).toBe(0.35);
    expect(get(outputSettings)).toEqual({
      mode: 'shared',
      policy: 'compatibility',
      fade: true,
      timing: 'event',
    });
    expect(getTrackByIdMock).toHaveBeenCalledWith(77);
    expect(loadAlbumTracksIfNeededMock).toHaveBeenCalledTimes(1);
  });
});
