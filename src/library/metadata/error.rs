use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum MetadataError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("ID3 tag error: {0}")]
    Id3(#[from] id3::Error),
}
