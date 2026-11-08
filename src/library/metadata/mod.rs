pub mod error;
pub mod id3;
pub mod read;
pub mod tags;
pub mod write;

pub use error::MetadataError;

pub use read::extract_track_metadata;
pub use tags::*;
pub use write::{SaveTrackArgs, save_track_info};
