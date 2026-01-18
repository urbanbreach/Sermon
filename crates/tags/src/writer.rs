//! Tag writing functionality using Lofty
//!
//! Provides a patch-based API for modifying audio file metadata.

use lofty::config::WriteOptions;
use lofty::file::TaggedFileExt;
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

/// Collection of patches to apply to a track's tags
#[derive(Debug, Clone, Default)]
pub struct TagPatches {
    pub title: TagPatch,
    pub artist: TagPatch,
    pub album: TagPatch,
    pub album_artist: TagPatch,
    pub genre: TagPatch,
    pub track_no: NumberPatch,
    pub disc_no: NumberPatch,
    pub year: NumberPatch,
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
        assert_eq!(patches.track_no, NumberPatch::Leave);
        assert_eq!(patches.disc_no, NumberPatch::Leave);
        assert_eq!(patches.year, NumberPatch::Leave);
    }
}
