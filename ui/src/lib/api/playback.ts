import { invoke } from '@tauri-apps/api/core';

export interface AudioDeviceInfo {
  id: string;
  name: string;
  is_default: boolean;
}

export interface AudioOutputSettings {
  mode: 'exclusive' | 'shared' | 'asio';
  policy: 'strict' | 'compatibility';
  fade: boolean;
  timing: 'event' | 'polling';
  asioDriver?: string;
}

export interface AsioDriverInfo {
  name: string;
}

export async function listAsioDrivers(): Promise<AsioDriverInfo[]> {
  return invoke('cmd_list_asio_drivers');
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
  return invoke('cmd_playback_seek', { positionMs: Math.floor(positionMs) });
}

export async function playbackNext(): Promise<void> {
  return invoke('cmd_playback_next');
}

export async function playbackPrevious(): Promise<void> {
  return invoke('cmd_playback_previous');
}

export async function restorePlaybackSession(): Promise<void> {
  await invoke('cmd_playback_restore_session');
}

export async function queuePlayNow(trackId: number): Promise<void> {
  return invoke('cmd_queue_play_now', { trackId });
}

export async function queueAdd(trackId: number): Promise<void> {
  return invoke('cmd_queue_add', { trackId });
}

export async function queueAddNext(trackIds: number[]): Promise<void> {
  return invoke('cmd_queue_add_next', { trackIds });
}

export async function queueSetAndPlay(trackIds: number[], startIndex: number): Promise<void> {
  return invoke('cmd_queue_set_and_play', { trackIds, startIndex });
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

export async function getOutputSettings(): Promise<AudioOutputSettings> {
  return invoke('cmd_output_get_settings');
}

export async function setOutputSettings(settings: AudioOutputSettings): Promise<void> {
  return invoke('cmd_output_set_settings', { settings });
}

export async function openAsioControlPanel(driverName: string): Promise<void> {
  return invoke('cmd_open_asio_control_panel', { driverName });
}

export interface ProbeCapabilitiesKey {
  backend: string;
  deviceId: string | null;
  asioDriver: string | null;
  channels: number;
}

export interface ProbeDimensions {
  sampleRates: number[];
  bitDepths: number[];
  channels: number[];
}

export interface ProbeCell {
  sampleRate: number;
  bitDepth: number;
  channels: number;
  supported: boolean;
  reasonCode: string;
  detail: string | null;
}

export interface ProbeCapabilitiesResult {
  version: number;
  key: ProbeCapabilitiesKey;
  probedAtMs: number;
  dimensions: ProbeDimensions;
  cells: ProbeCell[];
}

export async function probeOutputCapabilities(
  backend: 'wasapi' | 'asio',
  deviceId: string | null,
  asioDriver: string | null,
  channels: number,
  sampleRates: number[],
  bitDepths: number[]
): Promise<ProbeCapabilitiesResult> {
  return invoke('cmd_output_probe_capabilities', {
    backend,
    deviceId,
    asioDriver,
    channels,
    sampleRates,
    bitDepths,
  });
}
