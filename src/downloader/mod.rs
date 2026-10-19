mod complete;
mod parallel;
mod pipeline;
pub mod providers;
pub mod task;

pub use pipeline::run_download_pipeline;
pub use task::{Context, DownloadTask};
