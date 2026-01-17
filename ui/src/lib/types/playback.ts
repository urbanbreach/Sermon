export type PlaybackState = 'playing' | 'paused' | 'stopped';

export interface PlaybackStateEvent {
  state: PlaybackState;
  play_id: string | null;
  track_id: number | null;
}

export interface TrackEventData {
  id: number;
  title?: string;
  artist?: string;
  album?: string;
  duration_ms?: number;
  sample_rate?: number;
  bit_depth?: number;
  channels?: number;
  codec?: string;
  container?: string;
}

export interface NowPlayingEvent {
  play_id: string;
  track: TrackEventData;
  position_ms: number;
}

export interface PlaybackPositionEvent {
  play_id: string;
  position_ms: number;
  played_ms: number;
  duration_ms: number;
}

export interface QueueItemData {
  track_id: number;
  title?: string;
  artist?: string;
  album?: string;
  duration_ms?: number;
}

export interface QueueChangedEvent {
  play_id: string | null;
  current_index: number | null;
  queue: QueueItemData[];
}

export interface DeviceChangedEvent {
  device_id: string;
  device_name: string;
  is_default: boolean;
}

export interface AudioFormatData {
  sample_rate: number;
  bit_depth: number;
  channels: number;
  codec?: string;
  container?: string;
  valid_bits?: number;
}

export interface AudioDebugEvent {
  output_mode: 'exclusive' | 'shared';
  policy: 'strict' | 'compatibility';
  conversion: 'none' | 'shared_fallback' | 'pad_16_to_24';
  gain_mode: 'unity' | 'software';
  fade_enabled: boolean;
  exclusive_active: boolean;
  bit_perfect: 'yes' | 'no';
  bit_perfect_reason: string;
  device_id: string;
  device_name: string;
  output_format: AudioFormatData;
  decode_format: AudioFormatData;
}

export type PlaybackErrorAction = 'switch_to_default';

export interface PlaybackErrorEvent {
  code: string;
  message: string;
  track_id?: number;
  recoverable: boolean;
  action?: PlaybackErrorAction;
}
