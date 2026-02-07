pub mod artwork;
pub mod library;
pub mod lyrics;
pub mod playback;
pub mod settings;
pub mod waveform;

pub use artwork::{
    cache_exists, cmd_artwork_embed_to_file, cmd_artwork_extract_embedded, cmd_artwork_find_folder,
    cmd_artwork_get_best_for_album, cmd_artwork_get_best_for_track, cmd_artwork_get_bytes,
    cmd_artwork_get_thumb_bytes, cmd_artwork_search_candidates, cmd_artwork_select_candidate_for_album,
    generate_thumbnail, try_local_artwork,
};
pub use library::*;
pub use lyrics::cmd_lyrics_get_for_track;
pub use playback::*;
pub use settings::*;
pub use waveform::cmd_waveform_get_peaks;
