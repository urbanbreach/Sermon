mod reader;
pub mod writer;

pub use reader::{
    AudioMetadata, RawTagItem, RawTags, RawTagsResult, read_metadata, read_metadata_result,
    read_raw_tags,
};
pub use writer::{NumberPatch, TagPatch, TagPatches, TagWriteOptions, write_tags};
