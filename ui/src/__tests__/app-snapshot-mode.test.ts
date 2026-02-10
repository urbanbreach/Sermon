import { get } from 'svelte/store';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import appSource from '../App.svelte?raw';
import playbackSource from '../lib/state/playback.ts?raw';

describe('App snapshot-mode behavior', () => {
  beforeEach(() => {
    vi.unstubAllEnvs();
    vi.resetModules();
  });

  afterEach(() => {
    vi.unstubAllEnvs();
    vi.resetModules();
  });

  it('keeps snapshot-mode body class guard in App', () => {
    expect(appSource).toContain("import.meta.env.SERMON_SNAPSHOT === '1'");
    expect(appSource).toContain("document.body.classList.add('snapshot-mode')");
  });

  it('requires both SERMON_MOCK and SERMON_SNAPSHOT for snapshot playback seeding', () => {
    expect(playbackSource).toContain("import.meta.env.SERMON_MOCK === '1' && import.meta.env.SERMON_SNAPSHOT === '1'");
  });

  it('seeds playback deterministically when SERMON_MOCK and SERMON_SNAPSHOT are enabled', async () => {
    vi.stubEnv('SERMON_MOCK', '1');
    vi.stubEnv('SERMON_SNAPSHOT', '1');

    const eventApi = await import('@tauri-apps/api/event');
    const playback = await import('../lib/state/playback');

    playback.initPlaybackListeners();

    expect(eventApi.listen).not.toHaveBeenCalled();
    expect(get(playback.playbackState)).toBe('playing');
    expect(get(playback.positionMs)).toBe(0);
    expect(get(playback.queue)).toEqual([]);

    const seededTrack = get(playback.currentTrack);
    expect(seededTrack).not.toBeNull();
    expect(seededTrack?.id).toBe(0);
    expect(seededTrack?.title).toBeTruthy();
  });

  it('uses fixture-backed library data when SERMON_MOCK is enabled', async () => {
    vi.stubEnv('SERMON_MOCK', '1');

    const tauriCore = await import('@tauri-apps/api/core');
    const library = await import('../lib/api/library');
    const page = await library.listAlbumsPage(5);

    expect(page.items.length).toBeGreaterThan(0);
    expect(tauriCore.invoke).not.toHaveBeenCalled();
    expect(page.items[0]).toMatchObject({
      albumTitleDisplay: expect.any(String),
      albumArtistDisplay: expect.any(String),
      albumTitleSort: expect.any(String),
      albumArtistSort: expect.any(String),
    });
  });
});
