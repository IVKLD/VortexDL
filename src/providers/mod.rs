pub mod error;
pub mod hls;
pub mod soundcloud;
pub mod youtube;

use std::sync::Arc;

pub use error::ProviderError;
pub use hls::{ByteRange, HlsPlaylist, HlsSegment};
pub use soundcloud::SoundCloudProvider;
pub use youtube::YouTubeProvider;

pub type ProgressCallback = Arc<dyn Fn(u64, Option<u64>) + Send + Sync>;

#[derive(Debug, Clone)]
pub struct ResolvedStreamInfo {
    pub stream_url: String,
    pub proxy_used: Option<String>,
}
