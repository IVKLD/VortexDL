mod cancellation;
pub mod error;
pub mod queue;
pub mod service;
mod track;
pub mod types;

pub use cancellation::run_with_cancellation;
pub use error::DownloadError;
pub use queue::DownloadQueue;
pub use service::DownloadService;
pub use track::{DownloadedTrackInfo, TrackDownloader};
pub use types::{DownloadItem, DownloadStatus, DownloadTask, MessageLevel, ServerEvent};
