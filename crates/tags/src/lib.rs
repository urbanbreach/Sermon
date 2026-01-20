mod reader;
pub mod writer;

pub use reader::{
    ArtworkPicture, AudioMetadata, RawTagItem, RawTags, RawTagsResult, read_embedded_pictures,
    read_metadata, read_metadata_result, read_raw_tags,
};
pub use writer::{NumberPatch, PicturePatch, TagPatch, TagPatches, TagWriteOptions, write_tags};
