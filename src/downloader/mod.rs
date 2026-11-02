mod cancellation;
pub mod error;
pub mod manager;
pub mod pipeline;
pub mod types;
mod worker;

pub use cancellation::run_with_cancellation;
pub use error::DownloadError;
pub use manager::DownloadManager;
pub use pipeline::DownloadService;
pub use types::{DownloadItem, DownloadStatus, DownloadTask, MessageLevel, ServerEvent};
pub use worker::{DownloadedTrackInfo, TrackDownloader};
