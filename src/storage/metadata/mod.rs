pub mod id3;
pub mod read;
pub mod write;

pub use read::extract_track_metadata;
pub use write::{SaveTrackArgs, save_track_info};
