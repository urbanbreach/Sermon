export type PlaybackStateStr = 'playing' | 'paused' | 'stopped';
export type OutputModeStr = 'exclusive' | 'shared' | 'asio';
export type PolicyStr = 'strict' | 'compatibility';
export type EffectiveVolumeModeStr = 'unity_forced' | 'unity' | 'scaled';
export type SignalPathStatusStr = 'ok' | 'touching_bits' | 'unknown' | 'inactive';
export type BitPerfectStatusStr = 'yes' | 'no' | 'unknown';
export type DopIntegrityStatusStr = 'ok' | 'degraded' | 'unknown';
export type BackendKindStr = 'wasapi' | 'asio';

export interface TelemetryPlayback {
  state: PlaybackStateStr;
  track_id: number;
  is_dsd: boolean;
  output_mode: OutputModeStr;
  policy: PolicyStr;
  timing_mode: string;
  gain_mode: string;
  effective_volume_mode: EffectiveVolumeModeStr;
  fade_enabled: boolean;
  fade_active: boolean;
  conversion: string;
}

export interface TelemetryWasapiBackend {
  buffer_frames: number;
  device_period_default_hns: number;
  device_period_min_hns: number;
}

export interface TelemetryAsioBackend {
  driver_name: string;
  buffer_size_frames: number;
  sample_format: string;
  actual_sample_rate: number;
}

export interface TelemetryBackend {
  kind: BackendKindStr;
  wasapi?: TelemetryWasapiBackend;
  asio?: TelemetryAsioBackend;
}

export interface TelemetryDevice {
  device_id: string;
  device_name: string;
  exclusive_active: boolean;
  backend: TelemetryBackend;
}

export interface TelemetryDecodeFormat {
  sample_rate: number;
  bit_depth: number;
  channels: number;
  codec: string;
  container: string;
  is_dsd: boolean;
  dsd_rate_hz: number;
  dop_rate_hz: number;
}

export interface TelemetryOutputFormat {
  sample_rate: number;
  bit_depth: number;
  valid_bits: number;
  channels: number;
}

export interface TelemetryResamplerFormat {
  active: boolean;
  source_sample_rate: number;
  output_sample_rate: number;
}

export interface TelemetryFormat {
  decode: TelemetryDecodeFormat;
  output: TelemetryOutputFormat;
  resampler: TelemetryResamplerFormat;
}

export interface TelemetryCounter {
  track: number;
  lifetime: number;
}

export interface TelemetryRingBufferStats {
  capacity_frames: number;
  available_frames: number;
  fill_percent: number;
  underruns: TelemetryCounter;
  overflows: TelemetryCounter;
}

export interface TelemetryAsioStats {
  callback_underruns: TelemetryCounter;
  dop_drops: TelemetryCounter;
}

export interface TelemetryRecentEvent {
  ts_ms: number;
  code: string;
  detail: string;
}

export interface TelemetryStability {
  ring_buffer: TelemetryRingBufferStats;
  dop_ring_buffer: TelemetryRingBufferStats;
  asio: TelemetryAsioStats;
  recent_events: TelemetryRecentEvent[];
}

export interface TelemetryBitPerfect {
  status: BitPerfectStatusStr;
  reasons: string[];
  display: string;
}

export interface TelemetryDopIntegrity {
  status: DopIntegrityStatusStr;
  reasons: string[];
  display: string;
}

export interface TelemetryIntegrity {
  pcm_bit_perfect: TelemetryBitPerfect;
  dop_payload_integrity: TelemetryDopIntegrity;
}

export interface TelemetrySignalPathCheck {
  stage: string;
  status: SignalPathStatusStr;
  reason_code: string;
  detail: string;
}

export interface AudioTelemetryEvent {
  version: number;
  timestamp_ms: number;
  playback: TelemetryPlayback;
  device: TelemetryDevice;
  format: TelemetryFormat;
  stability: TelemetryStability;
  integrity: TelemetryIntegrity;
  signal_path_checks: TelemetrySignalPathCheck[];
}
