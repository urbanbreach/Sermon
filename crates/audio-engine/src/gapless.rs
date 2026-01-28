//! Gapless playback contract and primitives.
//!
//! This module defines the contract for gapless playback between tracks.
//! Gapless playback ensures seamless audio continuity when transitioning
//! from one track to another, with no audible gap or overlap.
//!
//! # The Gapless Contract
//!
//! The gapless playback contract specifies the following requirements:
//!
//! ## End-of-Track Detection
//!
//! A track is considered complete when the decoder's `next_packet()` method
//! returns `None`. This indicates that all audio data has been decoded and
//! there are no more packets available from the source.
//!
//! ## Sample Continuity
//!
//! Track B's first sample must immediately follow Track A's last *trimmed*
//! sample. "Trimmed" refers to samples after removing any encoder delay
//! (priming samples) and padding samples added by the encoder.
//!
//! For lossy codecs like MP3 and AAC:
//! - **Encoder delay (priming samples)**: The encoder adds silent samples at
//!   the beginning to "prime" the codec. These must be trimmed from playback.
//! - **Padding samples**: The encoder may add samples at the end to fill the
//!   final frame. These must also be trimmed.
//!
//! The metadata for trimming (encoder delay and padding) is typically stored in:
//! - MP3: LAME header, iTunes gapless info, or Xing header
//! - AAC: iTunes metadata atoms (iTunSMPB)
//! - FLAC: No encoder delay (sample-accurate by design)
//!
//! ## No Overlap, No Gap
//!
//! The contract requires **exact sample count** with zero tolerance:
//! - No overlapping samples between tracks
//! - No gap (missing samples) between tracks
//! - No crossfade or blending at the boundary
//!
//! This is the strictest form of gapless playback, suitable for:
//! - Live albums with continuous performances
//! - Concept albums with segues between tracks
//! - Classical music with connected movements
//!
//! # Usage
//!
//! ```rust
//! use audio_engine::gapless::GaplessContract;
//!
//! // Verify sample continuity tolerance is exact
//! assert_eq!(GaplessContract::SAMPLE_CONTINUITY_TOLERANCE, 0);
//! ```

/// Contract constants and documentation for gapless playback.
///
/// This struct serves as a namespace for gapless playback contract constants.
/// The constants define the exact requirements for seamless track transitions.
///
/// # Contract Semantics
///
/// ## End-of-Track Detection
///
/// A track is considered complete when the decoder returns `None` from
/// `next_packet()`. The decoder is responsible for:
/// 1. Decoding all packets from the audio source
/// 2. Returning `None` when no more packets are available
/// 3. Maintaining accurate sample position throughout decoding
///
/// ## Sample Continuity
///
/// For gapless playback to work correctly:
///
/// ```text
/// Track A: [sample_0, sample_1, ..., sample_N]  (after trimming)
/// Track B: [sample_0, sample_1, ..., sample_M]  (after trimming)
///
/// Output:  [..., A.sample_N, B.sample_0, ...]
///                          ^
///                          └── No gap, no overlap
/// ```
///
/// The transition point must satisfy:
/// - `A.sample_N` is the last valid audio sample from Track A (excluding padding)
/// - `B.sample_0` is the first valid audio sample from Track B (excluding encoder delay)
/// - There are exactly 0 missing or duplicated samples at the boundary
pub struct GaplessContract;

impl GaplessContract {
    /// Tolerance for sample continuity between tracks.
    ///
    /// A value of `0` means exact sample count is required:
    /// - No gap (missing samples) between tracks
    /// - No overlap (duplicated samples) between tracks
    ///
    /// This is the strictest gapless requirement, ensuring bit-perfect
    /// transitions for audiophile-quality playback.
    pub const SAMPLE_CONTINUITY_TOLERANCE: u64 = 0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gapless_contract_defined() {
        // Verify the contract constant exists and has the expected value
        assert_eq!(
            GaplessContract::SAMPLE_CONTINUITY_TOLERANCE,
            0,
            "Gapless contract requires exact sample continuity (zero tolerance)"
        );
    }

    #[test]
    fn test_gapless_contract_is_strict() {
        // The contract must enforce exact sample count with no tolerance
        // This ensures no gap and no overlap between tracks
        let tolerance = GaplessContract::SAMPLE_CONTINUITY_TOLERANCE;
        assert!(
            tolerance == 0,
            "Gapless playback must have zero tolerance for sample gaps/overlaps"
        );
    }
}
