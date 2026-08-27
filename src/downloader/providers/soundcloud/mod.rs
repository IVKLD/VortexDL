pub mod discovery;
pub mod download;
pub mod resolve;

pub use discovery::resolve_tracks_from_url;
pub use download::download_soundcloud_track;
pub use resolve::{resolve_stream_source, spawn_artwork_fetch};
