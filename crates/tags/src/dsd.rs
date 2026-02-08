use crate::reader::AudioMetadata;
use id3::TagLike;
use std::error::Error;
use std::fmt;
use std::path::Path;

#[derive(Debug)]
pub enum DsdError {
    Dsf(dsf::Error),
    Dff(dff_meta::model::Error),
}

impl fmt::Display for DsdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DsdError::Dsf(e) => write!(f, "DSF error: {}", e),
            DsdError::Dff(e) => write!(f, "DFF error: {}", e),
        }
    }
}

impl Error for DsdError {}

impl From<dsf::Error> for DsdError {
    fn from(e: dsf::Error) -> Self {
        DsdError::Dsf(e)
    }
}

impl From<dff_meta::model::Error> for DsdError {
    fn from(e: dff_meta::model::Error) -> Self {
        DsdError::Dff(e)
    }
}

pub fn read_dsf_metadata(path: &Path) -> Result<AudioMetadata, DsdError> {
    let dsf = dsf::DsfFile::open(path)?;
    let fmt = dsf.fmt_chunk();
    let dsd_rate_hz = fmt.sampling_frequency();
    let dsd_channels = fmt.channel_num() as u8;
    let sample_count = fmt.sample_count();
    let duration_ms = if dsd_rate_hz > 0 {
        Some((sample_count as u64 * 1000) / (dsd_rate_hz as u64))
    } else {
        None
    };

    let (title, artist, album, album_artist, track_no, disc_no, year, genre) =
        extract_id3_tags(dsf.id3_tag());

    Ok(AudioMetadata {
        title,
        artist,
        album,
        album_artist,
        track_no,
        disc_no,
        year,
        genre,
        lyrics: None,
        synced_lyrics: None,
        codec: Some("DSF".to_string()),
        container: Some("DSF".to_string()),
        sample_rate: Some(dsd_rate_hz),
        bit_depth: Some(1),
        channels: Some(dsd_channels),
        duration_ms,
        dsd_rate_hz: Some(dsd_rate_hz),
        dsd_channels: Some(dsd_channels),
    })
}

pub fn read_dff_metadata(path: &Path) -> Result<AudioMetadata, DsdError> {
    let dff = match dff_meta::DffFile::open(path) {
        Ok(f) => f,
        Err(dff_meta::model::Error::Id3Error(_e, partial_dff)) => partial_dff,
        Err(e) => return Err(e.into()),
    };

    let dsd_rate_hz = dff.get_sample_rate().unwrap_or(0);
    let dsd_channels = dff.get_num_channels().unwrap_or(0) as u8;
    let audio_bytes = dff.get_audio_length();
    let duration_ms = if dsd_rate_hz > 0 && dsd_channels > 0 {
        Some((audio_bytes * 8 * 1000) / (dsd_rate_hz as u64 * dsd_channels as u64))
    } else {
        None
    };

    let (title, artist, album, album_artist, track_no, disc_no, year, genre) =
        extract_id3_tags(dff.id3_tag());

    Ok(AudioMetadata {
        title,
        artist,
        album,
        album_artist,
        track_no,
        disc_no,
        year,
        genre,
        lyrics: None,
        synced_lyrics: None,
        codec: Some("DFF".to_string()),
        container: Some("DFF".to_string()),
        sample_rate: Some(dsd_rate_hz),
        bit_depth: Some(1),
        channels: Some(dsd_channels),
        duration_ms,
        dsd_rate_hz: Some(dsd_rate_hz),
        dsd_channels: Some(dsd_channels),
    })
}

fn extract_id3_tags(
    tag: &Option<id3::Tag>,
) -> (
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<u32>,
    Option<u32>,
    Option<u32>,
    Option<String>,
) {
    if let Some(t) = tag {
        (
            t.title().map(|s| s.to_string()),
            t.artist().map(|s| s.to_string()),
            t.album().map(|s| s.to_string()),
            t.album_artist().map(|s| s.to_string()),
            t.track(),
            t.disc(),
            t.year().map(|y| y as u32),
            t.genre_parsed().map(|g| g.to_string()),
        )
    } else {
        (None, None, None, None, None, None, None, None)
    }
}
