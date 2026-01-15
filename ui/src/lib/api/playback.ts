import { invoke } from '@tauri-apps/api/core';

export interface AudioDeviceInfo {
  id: string;
  name: string;
  is_default: boolean;
}

export async function playbackStart(trackId: number): Promise<void> {
  return invoke('cmd_playback_start', { trackId });
}

export async function playbackPause(): Promise<void> {
  return invoke('cmd_playback_pause');
}

export async function playbackResume(): Promise<void> {
  return invoke('cmd_playback_resume');
}

export async function playbackStop(): Promise<void> {
  return invoke('cmd_playback_stop');
}

export async function playbackSeek(positionMs: number): Promise<void> {
  return invoke('cmd_playback_seek', { positionMs });
}

export async function playbackNext(): Promise<void> {
  return invoke('cmd_playback_next');
}

export async function playbackPrevious(): Promise<void> {
  return invoke('cmd_playback_previous');
}

export async function queuePlayNow(trackId: number): Promise<void> {
  return invoke('cmd_queue_play_now', { trackId });
}

export async function queueAdd(trackId: number): Promise<void> {
  return invoke('cmd_queue_add', { trackId });
}

export async function listDevices(): Promise<AudioDeviceInfo[]> {
  return invoke('cmd_output_list_devices');
}

export async function setDevice(deviceId: string): Promise<void> {
  return invoke('cmd_output_set_device', { deviceId });
}

export async function getVolume(): Promise<number> {
  return invoke('cmd_volume_get');
}

export async function setVolume(volume: number): Promise<void> {
  return invoke('cmd_volume_set', { volume });
}
