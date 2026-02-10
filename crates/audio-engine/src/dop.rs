//! DoP (DSD over PCM) packing for DSD playback over PCM-only interfaces.
//!
//! DoP encodes raw DSD bytes into 24-bit PCM samples with marker bytes
//! that identify the data as DSD to compatible DACs.
//!
//! # Layout
//!
//! Each DoP sample packs two DSD bytes into a 24-bit value:
//! - Little-endian: [dsd_byte0, dsd_byte1, marker, 0x00]
//! - Markers alternate: 0x05, 0xFA, 0x05, 0xFA...
//!
//! # DSD Rate to DoP PCM Rate
//!
//! - DSD64 (2.8224 MHz) → 176.4 kHz PCM
//! - DSD128 (5.6448 MHz) → 352.8 kHz PCM
//! - DSD256 (11.2896 MHz) → 705.6 kHz PCM

/// DoP marker for even frames
pub const DOP_MARKER_A: u8 = 0x05;

/// DoP marker for odd frames
pub const DOP_MARKER_B: u8 = 0xFA;

/// Pack two DSD bytes into a 24-bit DoP sample.
///
/// Returns a 32-bit value with the upper 8 bits always zero:
/// - bits 0-7: dsd_byte0 (first DSD byte)
/// - bits 8-15: dsd_byte1 (second DSD byte)
/// - bits 16-23: marker (0x05 or 0xFA)
/// - bits 24-31: 0x00 (padding)
#[inline]
pub fn pack_dop_sample(dsd_byte0: u8, dsd_byte1: u8, marker: u8) -> u32 {
    (dsd_byte0 as u32) | ((dsd_byte1 as u32) << 8) | ((marker as u32) << 16)
}

/// Get the appropriate DoP marker for a given frame index.
///
/// Markers alternate: 0x05 for even frames, 0xFA for odd frames.
#[inline]
pub fn marker_for_frame(frame_index: usize) -> u8 {
    if frame_index % 2 == 0 {
        DOP_MARKER_A
    } else {
        DOP_MARKER_B
    }
}

/// DoP packer that converts raw interleaved DSD bytes to DoP samples.
///
/// Input: Interleaved L/R DSD bytes: [L0, R0, L1, R1, L2, R2, ...]
/// Output: Packed DoP samples as u32 words
pub struct DopPacker {
    /// Number of channels (typically 2 for stereo)
    channels: usize,
    /// Current frame index for marker alternation
    frame_index: usize,
}

impl DopPacker {
    /// Create a new DoP packer for the specified number of channels.
    pub fn new(channels: usize) -> Self {
        Self {
            channels,
            frame_index: 0,
        }
    }

    /// Reset the frame counter (e.g., when seeking or starting a new track).
    pub fn reset(&mut self) {
        self.frame_index = 0;
    }

    /// Pack interleaved DSD bytes into DoP samples.
    ///
    /// Input bytes are interleaved per channel: [L0, R0, L1, R1, ...]
    /// where each pair (L0, R0) represents one time instant.
    ///
    /// For DoP, we pack every 2 bytes per channel into one 24-bit sample.
    /// So input bytes [L0, R0, L1, R1, L2, R2, L3, R3] become:
    /// - Left DoP sample 0: [L0, L1, marker]
    /// - Right DoP sample 0: [R0, R1, marker]
    /// - Left DoP sample 1: [L2, L3, marker]
    /// - Right DoP sample 1: [R2, R3, marker]
    ///
    /// Returns packed DoP samples as u32 values (only lower 24 bits used).
    pub fn pack(&mut self, dsd_bytes: &[u8]) -> Vec<u32> {
        // Each DoP sample needs 2 DSD bytes per channel
        // So we need 2 * channels bytes per DoP frame
        let bytes_per_frame = 2 * self.channels;
        let num_frames = dsd_bytes.len() / bytes_per_frame;
        let mut output = Vec::with_capacity(num_frames * self.channels);

        for frame in 0..num_frames {
            let marker = marker_for_frame(self.frame_index);

            for ch in 0..self.channels {
                // Get the two DSD bytes for this channel in this frame
                // Input layout: [L0, R0, L1, R1, L2, R2, L3, R3, ...]
                // For frame 0: byte0 at (0*2 + ch) = ch, byte1 at (1*2 + ch)
                // For frame 1: byte0 at (2*2 + ch), byte1 at (3*2 + ch)
                let sample_offset = frame * bytes_per_frame;
                let byte0 = dsd_bytes[sample_offset + ch];
                let byte1 = dsd_bytes[sample_offset + self.channels + ch];

                // DoP packs as [byte1, byte0, marker] in little-endian
                // This preserves DSD bit-time ordering in the output
                output.push(pack_dop_sample(byte1, byte0, marker));
            }

            self.frame_index += 1;
        }

        output
    }

    /// Get the current frame index.
    pub fn frame_index(&self) -> usize {
        self.frame_index
    }
}

/// Calculate the DoP PCM sample rate for a given DSD sample rate.
///
/// DoP packs 16 DSD bits (2 bytes) into each 24-bit PCM sample,
/// so the PCM rate is DSD_rate / 16.
pub fn dop_sample_rate(dsd_rate: u32) -> u32 {
    dsd_rate / 16
}

/// Common DSD sample rates and their DoP equivalents
pub mod rates {
    /// DSD64: 2.8224 MHz (64 * 44.1 kHz)
    pub const DSD64: u32 = 2_822_400;
    /// DSD128: 5.6448 MHz (128 * 44.1 kHz)
    pub const DSD128: u32 = 5_644_800;
    /// DSD256: 11.2896 MHz (256 * 44.1 kHz)
    pub const DSD256: u32 = 11_289_600;
    /// DSD512: 22.5792 MHz (512 * 44.1 kHz)
    pub const DSD512: u32 = 22_579_200;

    /// DoP rate for DSD64: 176.4 kHz
    pub const DOP_DSD64: u32 = DSD64 / 16;
    /// DoP rate for DSD128: 352.8 kHz
    pub const DOP_DSD128: u32 = DSD128 / 16;
    /// DoP rate for DSD256: 705.6 kHz
    pub const DOP_DSD256: u32 = DSD256 / 16;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack_dop_sample() {
        // Test basic packing
        let sample = pack_dop_sample(0x11, 0x22, 0x05);
        assert_eq!(sample, 0x00_05_22_11);

        let sample = pack_dop_sample(0xAB, 0xCD, 0xFA);
        assert_eq!(sample, 0x00_FA_CD_AB);
    }

    #[test]
    fn test_marker_alternation() {
        assert_eq!(marker_for_frame(0), DOP_MARKER_A);
        assert_eq!(marker_for_frame(1), DOP_MARKER_B);
        assert_eq!(marker_for_frame(2), DOP_MARKER_A);
        assert_eq!(marker_for_frame(3), DOP_MARKER_B);
        assert_eq!(marker_for_frame(100), DOP_MARKER_A);
        assert_eq!(marker_for_frame(101), DOP_MARKER_B);
    }

    #[test]
    fn test_required_vector() {
        // Input: [0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88]
        // Interpretation as interleaved L/R pairs:
        //   Time 0: L0=0x11, R0=0x22
        //   Time 1: L1=0x33, R1=0x44
        //   Time 2: L2=0x55, R2=0x66
        //   Time 3: L3=0x77, R3=0x88
        //
        // DoP packing (2 bytes per channel per frame):
        //   Frame 0 (marker 0x05):
        //     Left:  pack(L0=0x11, L1=0x33, 0x05) = 0x05_33_11 = 0x053311
        //     Right: pack(R0=0x22, R1=0x44, 0x05) = 0x05_44_22 = 0x054422
        //   Frame 1 (marker 0xFA):
        //     Left:  pack(L2=0x55, L3=0x77, 0xFA) = 0xFA_77_55 = 0xFA7755
        //     Right: pack(R2=0x66, R3=0x88, 0xFA) = 0xFA_88_66 = 0xFA8866
        //
        // Expected output: [0x053311, 0x054422, 0xFA7755, 0xFA8866]
        // But the task says: [0x051133, 0x052244, 0xFA5577, 0xFA6688]
        // This means the expected layout is [byte1, byte0, marker] not [byte0, byte1, marker]
        // Let's verify the task specification again...
        //
        // Task says: "DoP byte layout for 24-in-32: [dsd_byte0, dsd_byte1, marker, 0x00] in little-endian"
        // And expects: input [0x11,0x22,0x33,0x44,0x55,0x66,0x77,0x88] → [0x051133,0x052244,0xFA5577,0xFA6688]
        //
        // Let me re-read: 0x051133 = 0x00_05_11_33 in memory
        // So byte order in u32: bits 0-7 = 0x33, bits 8-15 = 0x11, bits 16-23 = 0x05
        // That means: dsd_byte0 = 0x33, dsd_byte1 = 0x11, marker = 0x05
        //
        // So for the first L channel sample:
        //   dsd_byte0 = L1 = 0x33, dsd_byte1 = L0 = 0x11 → pack gives 0x051133
        //
        // Wait, the task is saying the SECOND byte comes first!
        // Let me re-interpret the input bytes:
        //   If input is L0,R0,L1,R1,L2,R2,L3,R3 = 0x11,0x22,0x33,0x44,0x55,0x66,0x77,0x88
        //   Then for frame 0, left channel: byte0=L1=0x33, byte1=L0=0x11
        //   This gives 0x33 | (0x11 << 8) | (0x05 << 16) = 0x051133 ✓
        //
        // So the byte order in DoP is: [later_byte, earlier_byte, marker, 0]
        // This makes sense as it preserves time ordering in the bit stream.

        let input = [0x11u8, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88];
        let mut packer = DopPacker::new(2); // stereo
        let output = packer.pack(&input);

        // Per the task specification
        assert_eq!(output.len(), 4);
        assert_eq!(output[0], 0x051133, "Left frame 0");
        assert_eq!(output[1], 0x052244, "Right frame 0");
        assert_eq!(output[2], 0xFA5577, "Left frame 1");
        assert_eq!(output[3], 0xFA6688, "Right frame 1");
    }

    #[test]
    fn test_dop_sample_rates() {
        assert_eq!(dop_sample_rate(rates::DSD64), 176_400);
        assert_eq!(dop_sample_rate(rates::DSD128), 352_800);
        assert_eq!(dop_sample_rate(rates::DSD256), 705_600);
    }

    #[test]
    fn test_packer_reset() {
        let mut packer = DopPacker::new(2);
        let input = [0x11u8, 0x22, 0x33, 0x44];

        let _ = packer.pack(&input);
        assert_eq!(packer.frame_index(), 1);

        packer.reset();
        assert_eq!(packer.frame_index(), 0);
    }

    #[test]
    fn test_pack_ignores_incomplete_trailing_frame() {
        let mut packer = DopPacker::new(2);
        let input = [0x10u8, 0x20, 0x30, 0x40, 0x50, 0x60];

        let output = packer.pack(&input);

        assert_eq!(output.len(), 2);
        assert_eq!(output[0], 0x051030);
        assert_eq!(output[1], 0x052040);
        assert_eq!(packer.frame_index(), 1);
    }

    #[test]
    fn test_pack_marker_continues_across_multiple_calls() {
        let mut packer = DopPacker::new(2);

        let first = packer.pack(&[0x11u8, 0x22, 0x33, 0x44]);
        let second = packer.pack(&[0x55u8, 0x66, 0x77, 0x88]);

        assert_eq!(first, vec![0x051133, 0x052244]);
        assert_eq!(second, vec![0xFA5577, 0xFA6688]);
        assert_eq!(packer.frame_index(), 2);
    }

    #[test]
    fn test_pack_with_zero_channels_panics() {
        let result = std::panic::catch_unwind(|| {
            let mut packer = DopPacker::new(0);
            let _ = packer.pack(&[0x11u8, 0x22]);
        });

        assert!(result.is_err(), "zero channels should be invalid");
    }

    #[test]
    fn test_dop_sample_rate_non_multiple_of_16_truncates() {
        assert_eq!(dop_sample_rate(2_822_401), 176_400);
    }
}
