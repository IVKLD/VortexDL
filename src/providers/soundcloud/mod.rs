pub mod client;
pub mod discovery;
pub mod download;
pub mod resolve;

pub use client::{
    ClientBuilder, SoundcloudExt, resize_artwork_url, resolve_soundcloud_stream_with_fallback,
    update_cached_client_id,
};
pub use discovery::resolve_tracks_from_url;
pub use download::download_soundcloud_track;
pub use resolve::{
    StreamSource, resolve_soundcloud_stream, resolve_stream_source, spawn_artwork_fetch,
};
