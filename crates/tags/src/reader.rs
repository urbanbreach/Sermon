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

/// A single raw tag item for display
#[derive(Debug, Clone)]
pub struct RawTagItem {
    pub key: String,
    pub value: String,
}

/// Raw tags from all tag types in a file
#[derive(Debug, Clone, Default)]
pub struct RawTags {
    pub tag_type: String,
    pub items: Vec<RawTagItem>,
}

/// All raw tags from a file (may have multiple tag types)
#[derive(Debug, Clone, Default)]
pub struct RawTagsResult {
    pub tags: Vec<RawTags>,
}

/// Embedded artwork picture from audio file tags
#[derive(Debug, Clone)]
pub struct ArtworkPicture {
    /// MIME type (e.g., "image/jpeg", "image/png")
    pub mime: Option<String>,
    /// Picture type (e.g., "CoverFront", "Other")
    pub picture_type: String,
    /// Raw image bytes
    pub bytes: Vec<u8>,
}

/// Read metadata from an audio file
/// Returns partial metadata on parse errors (non-fatal)
pub fn read_metadata(path: &Path) -> AudioMetadata {
    match read_metadata_result(path) {
        Ok(meta) => meta,
        Err(e) => {
            warn!("Failed to read metadata from {:?}: {}", path, e);
            AudioMetadata::default()
        }
    }
}

/// Read metadata from an audio file, returning errors explicitly
///
/// Use this when you need to distinguish between "no tags" and "failed to parse".
/// This is useful for post-write verification and DB sync.
pub fn read_metadata_result(path: &Path) -> Result<AudioMetadata, lofty::error::LoftyError> {
    read_metadata_inner(path)
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

/// Read all raw tags from a file for debugging/inspection
///
/// Returns all tag items from all tag types present in the file.
/// Useful for audiophiles who want to see exactly what's in their files.
pub fn read_raw_tags(path: &Path) -> Result<RawTagsResult, lofty::error::LoftyError> {
    let tagged_file = Probe::open(path)?.read()?;

    let mut result = RawTagsResult::default();

    for tag in tagged_file.tags() {
        let tag_type_name = format!("{:?}", tag.tag_type());
        let mut items = Vec::new();

        for item in tag.items() {
            let key = format!("{:?}", item.key());
            let value = match item.value() {
                lofty::tag::ItemValue::Text(s) => s.clone(),
                lofty::tag::ItemValue::Locator(s) => format!("[URL] {}", s),
                lofty::tag::ItemValue::Binary(b) => format!("[Binary: {} bytes]", b.len()),
            };
            items.push(RawTagItem { key, value });
        }

        result.tags.push(RawTags {
            tag_type: tag_type_name,
            items,
        });
    }

    Ok(result)
}

/// Read embedded pictures from an audio file
///
/// Returns all embedded pictures found in the file's tags.
/// Picture selection (which to use as album art) is done by the caller.
pub fn read_embedded_pictures(
    path: &Path,
) -> Result<Vec<ArtworkPicture>, lofty::error::LoftyError> {
    let tagged_file = Probe::open(path)?.read()?;
    let mut pictures = Vec::new();

    for tag in tagged_file.tags() {
        for picture in tag.pictures() {
            let mime = picture.mime_type().map(|m| m.to_string());
            let picture_type = format!("{:?}", picture.pic_type());
            let bytes = picture.data().to_vec();

            pictures.push(ArtworkPicture {
                mime,
                picture_type,
                bytes,
            });
        }
    }

    Ok(pictures)
}
