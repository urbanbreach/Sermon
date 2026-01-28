//! Encoder delay and padding detection for gapless playback.
//!
//! Lossy codecs like MP3 and AAC add silent samples at the beginning
//! (encoder delay/priming) and end (padding) of the encoded stream.
//! These must be trimmed for proper gapless playback.
//!
//! # Supported Formats
//!
//! ## MP3 (LAME Header)
//!
//! The LAME encoder stores gapless info in the Xing/Info header frame:
//! - Encoder delay (priming samples): 12 bits
//! - Encoder padding: 12 bits
//!
//! The delay/padding values are packed into 3 bytes at offset 0x15 from
//! the "LAME" signature as a big-endian 24-bit value.
//!
//! ## AAC (iTunSMPB)
//!
//! iTunes and compatible encoders store gapless info in the iTunSMPB
//! metadata tag as a space-separated hex string:
//!
//! ```text
//! " 00000000 XXXXXXXX YYYYYYYY ZZZZZZZZZZZZZZZZ ..."
//!            ^^^^^^^^ ^^^^^^^^
//!            delay    padding
//! ```
//!
//! # Usage
//!
//! ```rust,ignore
//! use audio_engine::encoder_delay::{parse_mp3_lame_header, parse_aac_itunes_smpb, EncoderDelay};
//!
//! // Parse LAME header from first MP3 frame
//! if let Some(delay) = parse_mp3_lame_header(&frame_data) {
//!     println!("Start skip: {} samples, End skip: {} samples",
//!              delay.start_samples, delay.end_samples);
//! }
//!
//! // Parse iTunSMPB string from AAC metadata
//! let smpb = " 00000000 00000A40 000002E8 0000000000123456";
//! if let Some(delay) = parse_aac_itunes_smpb(smpb) {
//!     println!("Start skip: {} samples, End skip: {} samples",
//!              delay.start_samples, delay.end_samples);
//! }
//! ```

/// Encoder delay and padding information for gapless playback.
///
/// These values represent the number of samples that should be trimmed
/// from the decoded audio to achieve gapless playback:
///
/// - `start_samples`: Samples to skip at the start (encoder delay/priming)
/// - `end_samples`: Samples to skip at the end (padding)
///
/// For proper gapless playback, a decoder should:
/// 1. Skip the first `start_samples` decoded samples
/// 2. Stop playback `end_samples` before the end of the decoded stream
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EncoderDelay {
    /// Number of samples to skip at the start of the stream (encoder delay/priming).
    pub start_samples: u64,
    /// Number of samples to skip at the end of the stream (padding).
    pub end_samples: u64,
}

impl EncoderDelay {
    /// Create a new EncoderDelay with the given values.
    pub fn new(start_samples: u64, end_samples: u64) -> Self {
        Self {
            start_samples,
            end_samples,
        }
    }

    /// Returns true if there is any delay or padding to trim.
    pub fn has_trimming(&self) -> bool {
        self.start_samples > 0 || self.end_samples > 0
    }

    /// Detect encoder delay from raw file data.
    ///
    /// This currently detects the MP3 LAME/Xing header from the first frame.
    ///
    /// # Arguments
    ///
    /// * `data` - Raw file data (at least first few KB for header detection)
    ///
    /// # Returns
    ///
    /// `Some(EncoderDelay)` if delay info was found, `None` otherwise.
    pub fn detect(data: &[u8]) -> Option<Self> {
        // Try MP3 LAME header (works on raw data)
        parse_mp3_lame_header(data)
    }
}

/// LAME tag signature
const LAME_SIGNATURE: &[u8; 4] = b"LAME";

/// Offset from LAME signature to delay/padding bytes
/// LAME tag structure:
/// - Bytes 0-8: Version string (e.g., "LAME3.99r")
/// - Bytes 9: Info tag revision + VBR method
/// - Bytes 10: Lowpass filter value
/// - Bytes 11-18: Replay Gain data (8 bytes)
/// - Bytes 19: Encoding flags
/// - Bytes 20: ABR/Minimum bitrate
/// - Bytes 21-23: Encoder delay (12 bits) + Padding (12 bits)
const LAME_DELAY_OFFSET: usize = 21;

/// Minimum size needed for LAME tag parsing (signature + delay/padding)
const LAME_MIN_SIZE: usize = LAME_DELAY_OFFSET + 3;

/// Parse encoder delay and padding from MP3 LAME/Xing header.
///
/// The LAME encoder stores gapless information in the first MP3 frame,
/// embedded within the Xing/Info VBR header. This function searches for
/// the "LAME" signature and extracts the delay/padding values.
///
/// # Format
///
/// The delay and padding are packed into 3 bytes (24 bits) at offset 0x15
/// from the LAME signature:
///
/// ```text
/// Byte 0: [D11 D10 D9 D8 D7 D6 D5 D4]  (upper 8 bits of delay)
/// Byte 1: [D3  D2  D1 D0 P11 P10 P9 P8] (lower 4 bits delay, upper 4 bits padding)
/// Byte 2: [P7  P6  P5 P4 P3  P2  P1 P0] (lower 8 bits of padding)
/// ```
///
/// - Encoder delay = upper 12 bits
/// - Encoder padding = lower 12 bits
///
/// # Arguments
///
/// * `data` - Raw file data containing the first MP3 frame
///
/// # Returns
///
/// `Some(EncoderDelay)` if a valid LAME header was found, `None` otherwise.
///
/// # Example
///
/// ```rust
/// use audio_engine::encoder_delay::parse_mp3_lame_header;
///
/// // Mock data with LAME signature at offset 0
/// // Delay = 576 (0x240), Padding = 1000 (0x3E8)
/// // Packed: 0x240 << 12 | 0x3E8 = 0x2403E8
/// let mut data = vec![0u8; 100];
/// data[0..4].copy_from_slice(b"LAME");
/// data[21] = 0x24; // Upper 8 bits of delay
/// data[22] = 0x03; // Lower 4 bits delay (0) + upper 4 bits padding (3)
/// data[23] = 0xE8; // Lower 8 bits of padding
///
/// let delay = parse_mp3_lame_header(&data).unwrap();
/// assert_eq!(delay.start_samples, 576);
/// assert_eq!(delay.end_samples, 1000);
/// ```
pub fn parse_mp3_lame_header(data: &[u8]) -> Option<EncoderDelay> {
    // Search for LAME signature in the first part of the file
    // The Xing header is typically within the first 1KB
    let search_limit = data.len().min(2048);

    // Find LAME signature
    let lame_pos = data[..search_limit]
        .windows(4)
        .position(|w| w == LAME_SIGNATURE)?;

    // Check if we have enough data after the signature
    if data.len() < lame_pos + LAME_MIN_SIZE {
        return None;
    }

    // Read the 3-byte delay/padding value as big-endian 24-bit
    let delay_offset = lame_pos + LAME_DELAY_OFFSET;
    let byte0 = data[delay_offset] as u32;
    let byte1 = data[delay_offset + 1] as u32;
    let byte2 = data[delay_offset + 2] as u32;

    let packed = (byte0 << 16) | (byte1 << 8) | byte2;

    // Extract delay (upper 12 bits) and padding (lower 12 bits)
    let encoder_delay = packed >> 12;
    let encoder_padding = packed & 0x0FFF;

    // Sanity check: values should be reasonable for MP3
    // MP3 frames are 1152 samples (MPEG-1) or 576 samples (MPEG-2)
    // Delay is typically 576-2257 samples, padding varies
    if encoder_delay > 10000 || encoder_padding > 10000 {
        return None;
    }

    // Skip entries with both values as zero (no gapless info)
    if encoder_delay == 0 && encoder_padding == 0 {
        return None;
    }

    Some(EncoderDelay::new(
        encoder_delay as u64,
        encoder_padding as u64,
    ))
}

/// Parse encoder delay and padding from AAC iTunSMPB metadata string.
///
/// iTunes and compatible encoders store gapless playback information in
/// the `iTunSMPB` metadata tag. The format is a space-separated string
/// of hexadecimal values.
///
/// # Format
///
/// ```text
/// " 00000000 XXXXXXXX YYYYYYYY ZZZZZZZZZZZZZZZZ 00000000 00000000 00000000 00000000"
///            ^^^^^^^  ^^^^^^^
///            delay    padding
/// ```
///
/// - Field 0: Always "00000000" (ignored)
/// - Field 1: Encoder delay (priming samples) as 8-digit hex
/// - Field 2: Padding samples as 8-digit hex
/// - Field 3: Original sample count as 16-digit hex (optional, ignored here)
///
/// # Arguments
///
/// * `metadata` - The iTunSMPB string value from metadata
///
/// # Returns
///
/// `Some(EncoderDelay)` if the string was successfully parsed, `None` otherwise.
///
/// # Example
///
/// ```rust
/// use audio_engine::encoder_delay::parse_aac_itunes_smpb;
///
/// let smpb = " 00000000 00000A40 000002E8 0000000000123456";
/// let delay = parse_aac_itunes_smpb(smpb).unwrap();
///
/// assert_eq!(delay.start_samples, 0x0A40); // 2624
/// assert_eq!(delay.end_samples, 0x02E8);   // 744
/// ```
pub fn parse_aac_itunes_smpb(metadata: &str) -> Option<EncoderDelay> {
    // Split on whitespace and filter empty strings
    let fields: Vec<&str> = metadata.split_whitespace().collect();

    // We need at least 3 fields: ignored, delay, padding
    if fields.len() < 3 {
        return None;
    }

    // Parse delay (field 1) and padding (field 2) as hex
    let encoder_delay = u64::from_str_radix(fields[1], 16).ok()?;
    let encoder_padding = u64::from_str_radix(fields[2], 16).ok()?;

    // Sanity check: values should be reasonable
    // AAC typically has delay around 1024-2624 samples
    if encoder_delay > 100_000 || encoder_padding > 100_000 {
        return None;
    }

    // Skip if both values are zero
    if encoder_delay == 0 && encoder_padding == 0 {
        return None;
    }

    Some(EncoderDelay::new(encoder_delay, encoder_padding))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encoder_delay_default() {
        let delay = EncoderDelay::default();
        assert_eq!(delay.start_samples, 0);
        assert_eq!(delay.end_samples, 0);
        assert!(!delay.has_trimming());
    }

    #[test]
    fn test_encoder_delay_new() {
        let delay = EncoderDelay::new(576, 1000);
        assert_eq!(delay.start_samples, 576);
        assert_eq!(delay.end_samples, 1000);
        assert!(delay.has_trimming());
    }

    #[test]
    fn test_encoder_delay_has_trimming() {
        assert!(EncoderDelay::new(1, 0).has_trimming());
        assert!(EncoderDelay::new(0, 1).has_trimming());
        assert!(EncoderDelay::new(100, 200).has_trimming());
        assert!(!EncoderDelay::new(0, 0).has_trimming());
    }

    // ==================== MP3 LAME Header Tests ====================

    #[test]
    fn test_parse_mp3_lame_header_basic() {
        // Create mock LAME header with known values
        // Delay = 576 (0x240), Padding = 1000 (0x3E8)
        // Packed: (0x240 << 12) | 0x3E8 = 0x2403E8
        let mut data = vec![0u8; 100];
        data[0..4].copy_from_slice(b"LAME");
        data[21] = 0x24; // byte0: upper 8 bits of delay
        data[22] = 0x03; // byte1: lower 4 bits delay (0) + upper 4 bits padding
        data[23] = 0xE8; // byte2: lower 8 bits of padding

        let delay = parse_mp3_lame_header(&data).unwrap();
        assert_eq!(delay.start_samples, 576); // 0x240
        assert_eq!(delay.end_samples, 1000); // 0x3E8
    }

    #[test]
    fn test_parse_mp3_lame_header_offset() {
        // LAME signature at offset 36 (typical for stereo MPEG1)
        let mut data = vec![0u8; 200];
        data[36..40].copy_from_slice(b"LAME");

        // Delay = 529 (0x211), Padding = 742 (0x2E6)
        // Packed: (0x211 << 12) | 0x2E6 = 0x2112E6
        data[36 + 21] = 0x21;
        data[36 + 22] = 0x12;
        data[36 + 23] = 0xE6;

        let delay = parse_mp3_lame_header(&data).unwrap();
        assert_eq!(delay.start_samples, 529);
        assert_eq!(delay.end_samples, 742);
    }

    #[test]
    fn test_parse_mp3_lame_header_typical_lame_values() {
        // Typical LAME values: delay=576, padding varies
        let mut data = vec![0u8; 100];
        data[0..4].copy_from_slice(b"LAME");

        // Delay = 576 (0x240), Padding = 0 (0x000)
        // Packed: 0x240000
        data[21] = 0x24;
        data[22] = 0x00;
        data[23] = 0x00;

        let delay = parse_mp3_lame_header(&data).unwrap();
        assert_eq!(delay.start_samples, 576);
        assert_eq!(delay.end_samples, 0);
    }

    #[test]
    fn test_parse_mp3_lame_header_no_signature() {
        let data = vec![0u8; 100];
        assert!(parse_mp3_lame_header(&data).is_none());
    }

    #[test]
    fn test_parse_mp3_lame_header_too_short() {
        // LAME signature but not enough data for delay/padding
        let mut data = vec![0u8; 20];
        data[0..4].copy_from_slice(b"LAME");
        assert!(parse_mp3_lame_header(&data).is_none());
    }

    #[test]
    fn test_parse_mp3_lame_header_zero_values_returns_none() {
        // Both delay and padding are zero - no gapless info
        let mut data = vec![0u8; 100];
        data[0..4].copy_from_slice(b"LAME");
        data[21] = 0x00;
        data[22] = 0x00;
        data[23] = 0x00;

        assert!(parse_mp3_lame_header(&data).is_none());
    }

    #[test]
    fn test_parse_mp3_lame_header_max_valid_values() {
        // Maximum 12-bit values: 4095 (0xFFF) for both
        let mut data = vec![0u8; 100];
        data[0..4].copy_from_slice(b"LAME");

        // Packed: (0xFFF << 12) | 0xFFF = 0xFFFFFF
        data[21] = 0xFF;
        data[22] = 0xFF;
        data[23] = 0xFF;

        let delay = parse_mp3_lame_header(&data).unwrap();
        assert_eq!(delay.start_samples, 4095);
        assert_eq!(delay.end_samples, 4095);
    }

    // ==================== AAC iTunSMPB Tests ====================

    #[test]
    fn test_parse_aac_itunes_smpb_basic() {
        let smpb = " 00000000 00000A40 000002E8 0000000000123456";
        let delay = parse_aac_itunes_smpb(smpb).unwrap();

        assert_eq!(delay.start_samples, 0x0A40); // 2624
        assert_eq!(delay.end_samples, 0x02E8); // 744
    }

    #[test]
    fn test_parse_aac_itunes_smpb_no_leading_space() {
        let smpb = "00000000 00000A40 000002E8 0000000000123456";
        let delay = parse_aac_itunes_smpb(smpb).unwrap();

        assert_eq!(delay.start_samples, 2624);
        assert_eq!(delay.end_samples, 744);
    }

    #[test]
    fn test_parse_aac_itunes_smpb_extra_fields() {
        // Full iTunSMPB format has 12 fields
        let smpb = " 00000000 00000400 00000300 0000000000500000 00000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000";
        let delay = parse_aac_itunes_smpb(smpb).unwrap();

        assert_eq!(delay.start_samples, 0x400); // 1024
        assert_eq!(delay.end_samples, 0x300); // 768
    }

    #[test]
    fn test_parse_aac_itunes_smpb_typical_itunes_values() {
        // Typical iTunes AAC values
        let smpb = " 00000000 00000A40 000002D0 0000000001234567";
        let delay = parse_aac_itunes_smpb(smpb).unwrap();

        assert_eq!(delay.start_samples, 2624); // Typical iTunes delay
        assert_eq!(delay.end_samples, 720);
    }

    #[test]
    fn test_parse_aac_itunes_smpb_minimum_fields() {
        // Only required fields (first is ignored)
        let smpb = "00000000 00000400 00000200";
        let delay = parse_aac_itunes_smpb(smpb).unwrap();

        assert_eq!(delay.start_samples, 1024);
        assert_eq!(delay.end_samples, 512);
    }

    #[test]
    fn test_parse_aac_itunes_smpb_too_few_fields() {
        let smpb = "00000000 00000A40";
        assert!(parse_aac_itunes_smpb(smpb).is_none());
    }

    #[test]
    fn test_parse_aac_itunes_smpb_invalid_hex() {
        let smpb = "00000000 GGGGGGGG 000002E8 0000000000123456";
        assert!(parse_aac_itunes_smpb(smpb).is_none());
    }

    #[test]
    fn test_parse_aac_itunes_smpb_zero_values() {
        let smpb = "00000000 00000000 00000000 0000000000000000";
        assert!(parse_aac_itunes_smpb(smpb).is_none());
    }

    #[test]
    fn test_parse_aac_itunes_smpb_empty_string() {
        assert!(parse_aac_itunes_smpb("").is_none());
    }

    #[test]
    fn test_parse_aac_itunes_smpb_lowercase_hex() {
        let smpb = " 00000000 00000a40 000002e8 0000000000123456";
        let delay = parse_aac_itunes_smpb(smpb).unwrap();

        assert_eq!(delay.start_samples, 2624);
        assert_eq!(delay.end_samples, 744);
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_encoder_delay_equality() {
        let a = EncoderDelay::new(100, 200);
        let b = EncoderDelay::new(100, 200);
        let c = EncoderDelay::new(100, 300);

        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn test_encoder_delay_debug() {
        let delay = EncoderDelay::new(576, 1000);
        let debug_str = format!("{:?}", delay);
        assert!(debug_str.contains("576"));
        assert!(debug_str.contains("1000"));
    }

    #[test]
    fn test_encoder_delay_clone() {
        let delay = EncoderDelay::new(576, 1000);
        let cloned = delay;
        assert_eq!(delay, cloned);
    }
}
