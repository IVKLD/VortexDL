pub mod discovery;
pub mod download;
pub mod resolve;

pub use discovery::discover_youtube_tracks;
pub use download::download_youtube_track;
pub use resolve::resolve_youtube_stream;
