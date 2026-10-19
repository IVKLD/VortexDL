use std::path::Path;

use anyhow::Result;
use id3::{
    Content, Error as Id3Error, ErrorKind as Id3ErrorKind, Tag, TagLike,
    frame::{ExtendedText, Frame},
};

pub fn get_tag(path: impl AsRef<Path>) -> Result<Tag> {
    let path = path.as_ref();
    match Tag::read_from_path(path) {
        Ok(tag) => Ok(tag),
        Err(Id3Error {
            kind: Id3ErrorKind::NoTag,
            ..
        }) => Ok(Tag::new()),
        Err(e) => {
            tracing::warn!(
                path = %path.display(),
                error = %e,
                "Failed to read ID3 tag, starting fresh",
            );
            Ok(Tag::new())
        }
    }
}

pub fn set_txxx(tag: &mut Tag, key: &str, value: &str) {
    tag.remove_extended_text(Some(key), None);
    tag.add_frame(Frame::with_content(
        "TXXX",
        Content::ExtendedText(ExtendedText {
            description: key.to_string(),
            value: value.to_string(),
        }),
    ));
}

pub fn get_txxx(tag: &Tag, key: &str) -> Option<String> {
    tag.extended_texts()
        .find(|f| f.description == key)
        .map(|f| f.value.clone())
}

pub fn get_txxx_with_legacy(tag: &Tag, key: &str, legacy_key: &str) -> Option<String> {
    get_txxx(tag, key).or_else(|| get_txxx(tag, legacy_key))
}
