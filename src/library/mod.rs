mod index;
pub mod metadata;
mod scanner;
pub mod service;
mod types;

pub use index::Library;
pub use service::{LibraryService, RegisterDownloadedTrackArgs};
pub use types::{LibraryStats, LibraryTrack};

pub const ARCHIVE_DIR: &str = ".archive";
