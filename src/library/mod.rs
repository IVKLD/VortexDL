pub mod index;
pub mod metadata;
pub mod scanner;
pub mod types;

pub const ARCHIVE_DIR: &str = "Archive";

pub use index::Library;
pub use scanner::scan_directory;
pub use types::LibraryTrack;
