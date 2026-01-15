use lofty::file::{AudioFile, TaggedFileExt};
use lofty::probe::Probe;
use lofty::tag::Accessor;
use std::path::Path;
use tracing::warn;

/// Metadata extracted from an audio file
#[derive(Debug, Clone, Default)]
pub struct AudioMetadata {
    // Tags
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub track_no: Option<u32>,
    pub disc_no: Option<u32>,
    pub year: Option<u32>,
    pub genre: Option<String>,
    // Technical
    pub codec: Option<String>,
    pub container: Option<String>,
    pub sample_rate: Option<u32>,
    pub bit_depth: Option<u8>,
    pub channels: Option<u8>,
    pub duration_ms: Option<u64>,
}

/// Read metadata from an audio file
/// Returns partial metadata on parse errors (non-fatal)
pub fn read_metadata(path: &Path) -> AudioMetadata {
    match read_metadata_inner(path) {
        Ok(meta) => meta,
        Err(e) => {
            warn!("Failed to read metadata from {:?}: {}", path, e);
            AudioMetadata::default()
        }
    }
}

fn read_metadata_inner(path: &Path) -> Result<AudioMetadata, lofty::error::LoftyError> {
    let tagged_file = Probe::open(path)?.read()?;

    // Get primary tag (prefer ID3v2/Vorbis, fall back to others)
    let tag = tagged_file
        .primary_tag()
        .or_else(|| tagged_file.first_tag());

    let properties = tagged_file.properties();

    // Extract technical info
    let codec = Some(format!("{:?}", tagged_file.file_type()));
    // For container, lofty doesn't expose it directly in a generic way easily differently than file_type sometimes,
    // but file_type usually gives format info (e.g. MP3, FLAC).
    // The prompt suggested: let container = Some(format!("{:?}", tagged_file.file_type()));
    // I will stick to that for now.
    let container = Some(format!("{:?}", tagged_file.file_type()));
    let sample_rate = properties.sample_rate();
    let bit_depth = properties.bit_depth();
    let channels = properties.channels();
    let duration_ms = Some(properties.duration().as_millis() as u64);

    // Extract tags
    let (title, artist, album, album_artist, track_no, disc_no, year, genre) = if let Some(t) = tag
    {
        (
            t.title().map(|s| s.to_string()),
            t.artist().map(|s| s.to_string()),
            t.album().map(|s| s.to_string()),
            t.get_string(&lofty::tag::ItemKey::AlbumArtist)
                .map(|s| s.to_string()),
            t.track(),
            t.disk(),
            t.year(),
            t.genre().map(|s| s.to_string()),
        )
    } else {
        (None, None, None, None, None, None, None, None)
    };

    Ok(AudioMetadata {
        title,
        artist,
        album,
        album_artist,
        track_no,
        disc_no,
        year,
        genre,
        codec,
        container,
        sample_rate,
        bit_depth,
        channels,
        duration_ms,
    })
}
