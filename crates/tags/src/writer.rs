//! Tag writing functionality using Lofty
//!
//! Provides a patch-based API for modifying audio file metadata.

use lofty::config::WriteOptions;
use lofty::file::TaggedFileExt;
use lofty::picture::{MimeType, Picture, PictureType};
use lofty::probe::Probe;
use lofty::tag::{Accessor, ItemKey, Tag, TagExt};
use std::fs::OpenOptions;
use std::path::Path;

/// Patch operation for string tag fields
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TagPatch {
    /// Leave the field unchanged
    Leave,
    /// Set the field to a new value
    Set(String),
    /// Remove/clear the field entirely
    Clear,
}

impl Default for TagPatch {
    fn default() -> Self {
        TagPatch::Leave
    }
}

/// Patch operation for numeric tag fields
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NumberPatch {
    /// Leave the field unchanged
    Leave,
    /// Set the field to a new value
    Set(u32),
    /// Remove/clear the field entirely
    Clear,
}

impl Default for NumberPatch {
    fn default() -> Self {
        NumberPatch::Leave
    }
}

/// Patch operation for cover art picture
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PicturePatch {
    /// Leave pictures unchanged
    Leave,
    /// Set cover art (replaces existing front cover if any)
    SetCover {
        /// Raw image bytes
        bytes: Vec<u8>,
        /// MIME type (e.g., "image/jpeg", "image/png")
        mime: String,
    },
    /// Remove all pictures
    ClearAll,
}

impl Default for PicturePatch {
    fn default() -> Self {
        PicturePatch::Leave
    }
}

/// Collection of patches to apply to a track's tags
#[derive(Debug, Clone, Default)]
pub struct TagPatches {
    pub title: TagPatch,
    pub artist: TagPatch,
    pub album: TagPatch,
    pub album_artist: TagPatch,
    pub genre: TagPatch,
    pub publisher: TagPatch,
    pub composer: TagPatch,
    pub conductor: TagPatch,
    pub comments: TagPatch,
    pub grouping: TagPatch,
    pub lyricist: TagPatch,
    pub plain_lyrics: TagPatch,
    pub synced_lyrics: TagPatch,
    pub track_no: NumberPatch,
    pub disc_no: NumberPatch,
    pub year: NumberPatch,
    pub picture: PicturePatch,
}

/// Options for tag writing (reserved for future use)
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct TagWriteOptions {}

impl TagWriteOptions {
    /// Create new default options
    pub fn new() -> Self {
        Self {}
    }
}

/// Write tag patches to an audio file
///
/// This function modifies the existing primary tag in-place to maximize
/// preservation of unknown fields. If no tag exists, a new one is created
/// using the file's primary tag type.
///
/// # Arguments
/// * `path` - Path to the audio file to modify (can be a temp file with non-audio extension)
/// * `patches` - The tag patches to apply
/// * `_options` - Write options (reserved for future use)
///
/// # Returns
/// * `Ok(())` on success
/// * `Err(LoftyError)` on failure (file not found, unsupported format, etc.)
pub fn write_tags(
    path: &Path,
    patches: &TagPatches,
    _options: &TagWriteOptions,
) -> Result<(), lofty::error::LoftyError> {
    // Read the file and extract/modify the tag, then drop the file handle
    let (tag, _file_type) = {
        // Open and parse the file - use guess_file_type() to detect format from content
        // rather than relying on file extension (important for .sermon-tmp files)
        let mut tagged_file = Probe::open(path)?.guess_file_type()?.read()?;

        // Get the file type for later use when saving
        let file_type = tagged_file.file_type();

        // Get the primary tag type for this file format
        let tag_type = tagged_file.primary_tag_type();

        // Get mutable reference to existing primary tag, or create a new one
        let tag = if let Some(tag) = tagged_file.primary_tag_mut() {
            tag
        } else {
            // No primary tag exists, create and insert one
            let new_tag = Tag::new(tag_type);
            tagged_file.insert_tag(new_tag);
            tagged_file
                .primary_tag_mut()
                .expect("just inserted primary tag")
        };

        // Apply string field patches
        apply_string_patch(
            tag,
            &patches.title,
            |t, v| t.set_title(v.into()),
            |t| t.remove_title(),
        );
        apply_string_patch(
            tag,
            &patches.artist,
            |t, v| t.set_artist(v.into()),
            |t| t.remove_artist(),
        );
        apply_string_patch(
            tag,
            &patches.album,
            |t, v| t.set_album(v.into()),
            |t| t.remove_album(),
        );
        apply_string_patch(
            tag,
            &patches.genre,
            |t, v| t.set_genre(v.into()),
            |t| t.remove_genre(),
        );

        // Album artist uses ItemKey API
        match &patches.album_artist {
            TagPatch::Leave => {}
            TagPatch::Set(value) => {
                tag.insert_text(ItemKey::AlbumArtist, value.clone());
            }
            TagPatch::Clear => {
                tag.remove_key(&ItemKey::AlbumArtist);
            }
        }

        match &patches.publisher {
            TagPatch::Leave => {}
            TagPatch::Set(value) => {
                tag.insert_text(ItemKey::Publisher, value.clone());
            }
            TagPatch::Clear => {
                tag.remove_key(&ItemKey::Publisher);
            }
        }

        match &patches.composer {
            TagPatch::Leave => {}
            TagPatch::Set(value) => {
                tag.insert_text(ItemKey::Composer, value.clone());
            }
            TagPatch::Clear => {
                tag.remove_key(&ItemKey::Composer);
            }
        }

        match &patches.conductor {
            TagPatch::Leave => {}
            TagPatch::Set(value) => {
                tag.insert_text(ItemKey::Conductor, value.clone());
            }
            TagPatch::Clear => {
                tag.remove_key(&ItemKey::Conductor);
            }
        }

        match &patches.comments {
            TagPatch::Leave => {}
            TagPatch::Set(value) => {
                tag.insert_text(ItemKey::Comment, value.clone());
            }
            TagPatch::Clear => {
                tag.remove_key(&ItemKey::Comment);
            }
        }

        match &patches.grouping {
            TagPatch::Leave => {}
            TagPatch::Set(value) => {
                tag.insert_text(ItemKey::ContentGroup, value.clone());
            }
            TagPatch::Clear => {
                tag.remove_key(&ItemKey::ContentGroup);
            }
        }

        // Lyricist and lyrics use ItemKey API
        match &patches.lyricist {
            TagPatch::Leave => {}
            TagPatch::Set(value) => {
                tag.insert_text(ItemKey::Lyricist, value.clone());
            }
            TagPatch::Clear => {
                tag.remove_key(&ItemKey::Lyricist);
            }
        }

        match merge_lyrics_patches(&patches.plain_lyrics, &patches.synced_lyrics) {
            TagPatch::Leave => {}
            TagPatch::Set(value) => {
                tag.insert_text(ItemKey::Lyrics, value);
            }
            TagPatch::Clear => {
                tag.remove_key(&ItemKey::Lyrics);
            }
        }

        // Apply numeric field patches
        apply_number_patch(
            tag,
            &patches.track_no,
            |t, v| t.set_track(v),
            |t| t.remove_track(),
        );
        apply_number_patch(
            tag,
            &patches.disc_no,
            |t, v| t.set_disk(v),
            |t| t.remove_disk(),
        );
        apply_number_patch(
            tag,
            &patches.year,
            |t, v| t.set_year(v),
            |t| t.remove_year(),
        );

        // Apply picture patch
        match &patches.picture {
            PicturePatch::Leave => {}
            PicturePatch::SetCover { bytes, mime } => {
                // Remove existing front cover pictures first
                tag.remove_picture_type(PictureType::CoverFront);

                // Parse MIME type
                let mime_type = match mime.as_str() {
                    "image/jpeg" => MimeType::Jpeg,
                    "image/png" => MimeType::Png,
                    "image/gif" => MimeType::Gif,
                    "image/bmp" => MimeType::Bmp,
                    "image/tiff" => MimeType::Tiff,
                    _ => MimeType::Unknown(mime.clone()),
                };

                // Create and add the new picture
                let picture = Picture::new_unchecked(
                    PictureType::CoverFront,
                    Some(mime_type),
                    None, // No description
                    bytes.clone(),
                );
                tag.push_picture(picture);
            }
            PicturePatch::ClearAll => {
                // Remove all pictures
                while tag.picture_count() > 0 {
                    tag.remove_picture(0);
                }
            }
        }

        // Clone the tag so we can drop tagged_file (which holds the file handle)
        (tag.clone(), file_type)
    }; // tagged_file is dropped here, releasing the file handle

    // Now save with the file handle released
    // Use save_to with a file handle - this avoids extension-based format detection
    let write_options = WriteOptions::new().remove_others(false);
    let mut file = OpenOptions::new().read(true).write(true).open(path)?;
    tag.save_to(&mut file, write_options)?;

    Ok(())
}

/// Helper to apply a string patch to a tag field
fn apply_string_patch<S, C>(tag: &mut Tag, patch: &TagPatch, setter: S, clearer: C)
where
    S: FnOnce(&mut Tag, &str),
    C: FnOnce(&mut Tag),
{
    match patch {
        TagPatch::Leave => {}
        TagPatch::Set(value) => setter(tag, value),
        TagPatch::Clear => clearer(tag),
    }
}

/// Helper to apply a numeric patch to a tag field
fn apply_number_patch<S, C>(tag: &mut Tag, patch: &NumberPatch, setter: S, clearer: C)
where
    S: FnOnce(&mut Tag, u32),
    C: FnOnce(&mut Tag),
{
    match patch {
        NumberPatch::Leave => {}
        NumberPatch::Set(value) => setter(tag, *value),
        NumberPatch::Clear => clearer(tag),
    }
}

fn merge_lyrics_patches(plain: &TagPatch, synced: &TagPatch) -> TagPatch {
    match (plain, synced) {
        (_, TagPatch::Set(value)) => TagPatch::Set(value.clone()),
        (TagPatch::Set(value), _) => TagPatch::Set(value.clone()),
        (TagPatch::Clear, _) | (_, TagPatch::Clear) => TagPatch::Clear,
        _ => TagPatch::Leave,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_patch_default() {
        assert_eq!(TagPatch::default(), TagPatch::Leave);
    }

    #[test]
    fn test_number_patch_default() {
        assert_eq!(NumberPatch::default(), NumberPatch::Leave);
    }

    #[test]
    fn test_tag_patches_default() {
        let patches = TagPatches::default();
        assert_eq!(patches.title, TagPatch::Leave);
        assert_eq!(patches.artist, TagPatch::Leave);
        assert_eq!(patches.album, TagPatch::Leave);
        assert_eq!(patches.album_artist, TagPatch::Leave);
        assert_eq!(patches.genre, TagPatch::Leave);
        assert_eq!(patches.publisher, TagPatch::Leave);
        assert_eq!(patches.composer, TagPatch::Leave);
        assert_eq!(patches.conductor, TagPatch::Leave);
        assert_eq!(patches.comments, TagPatch::Leave);
        assert_eq!(patches.grouping, TagPatch::Leave);
        assert_eq!(patches.lyricist, TagPatch::Leave);
        assert_eq!(patches.plain_lyrics, TagPatch::Leave);
        assert_eq!(patches.synced_lyrics, TagPatch::Leave);
        assert_eq!(patches.track_no, NumberPatch::Leave);
        assert_eq!(patches.disc_no, NumberPatch::Leave);
        assert_eq!(patches.year, NumberPatch::Leave);
    }

    #[test]
    fn test_merge_lyrics_patches_prefers_synced_set() {
        let plain = TagPatch::Set("plain".to_string());
        let synced = TagPatch::Set("[00:01.00]synced".to_string());
        assert_eq!(
            merge_lyrics_patches(&plain, &synced),
            TagPatch::Set("[00:01.00]synced".to_string())
        );
    }

    #[test]
    fn test_merge_lyrics_patches_uses_plain_when_synced_not_set() {
        let plain = TagPatch::Set("plain".to_string());
        assert_eq!(
            merge_lyrics_patches(&plain, &TagPatch::Leave),
            TagPatch::Set("plain".to_string())
        );
    }

    #[test]
    fn test_merge_lyrics_patches_clear_when_no_set() {
        assert_eq!(
            merge_lyrics_patches(&TagPatch::Clear, &TagPatch::Leave),
            TagPatch::Clear
        );
        assert_eq!(
            merge_lyrics_patches(&TagPatch::Leave, &TagPatch::Clear),
            TagPatch::Clear
        );
    }
}
