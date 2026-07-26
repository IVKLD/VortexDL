use std::path::Path;

use anyhow::Result;
use id3::{
    Content, Error as Id3Error, ErrorKind as Id3ErrorKind, Tag, TagLike, Version,
    frame::{ExtendedText, Frame, Picture, PictureType},
};
use url::Url;

use crate::{
    constants::{SC_ARTWORK_URL, SC_IDENTIFIER, SC_SOURCE_URL},
    types::TrackMetadata,
};

fn get_tag(path: impl AsRef<Path>) -> Result<Tag> {
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

fn set_txxx(tag: &mut Tag, key: &str, value: &str) {
    tag.remove_extended_text(Some(key), None);
    tag.add_frame(Frame::with_content(
        "TXXX",
        Content::ExtendedText(ExtendedText {
            description: key.to_string(),
            value: value.to_string(),
        }),
    ));
}

pub struct SaveTrackArgs<'a> {
    pub path: &'a Path,
    pub sc_id: &'a str,
    pub title: &'a str,
    pub artist: &'a str,
    pub artwork_url: Option<&'a Url>,
    pub source_url: Option<&'a Url>,
    pub artwork_data: Option<Vec<u8>>,
}

pub fn save_track_info(args: SaveTrackArgs) -> Result<()> {
    let mut tag = get_tag(args.path)?;

    tag.set_title(args.title);
    tag.set_artist(args.artist);

    set_txxx(&mut tag, SC_IDENTIFIER, args.sc_id);

    if let Some(url) = args.artwork_url {
        set_txxx(&mut tag, SC_ARTWORK_URL, url.as_str());
    }
    if let Some(url) = args.source_url {
        set_txxx(&mut tag, SC_SOURCE_URL, url.as_str());
    }

    if let Some(data) = args.artwork_data {
        tag.remove_all_pictures();
        tag.add_frame(Frame::with_content(
            "APIC",
            Content::Picture(Picture {
                mime_type: "image/jpeg".to_string(),
                picture_type: PictureType::CoverFront,
                description: "Cover".to_string(),
                data,
            }),
        ));
    }

    tag.write_to_path(args.path, Version::Id3v23)?;
    Ok(())
}

fn get_txxx(tag: &Tag, key: &str) -> Option<String> {
    tag.extended_texts()
        .find(|f| f.description == key)
        .map(|f| f.value.clone())
}

pub fn extract_track_metadata(path: impl AsRef<Path>) -> Option<TrackMetadata> {
    let path = path.as_ref();
    let tag = Tag::read_from_path(path).ok();

    let id = tag
        .as_ref()
        .and_then(|t| get_txxx(t, SC_IDENTIFIER))
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or_else(|| {
            use std::{
                collections::hash_map::DefaultHasher,
                hash::{Hash, Hasher},
            };

            let mut hasher = DefaultHasher::new();
            path.to_string_lossy().hash(&mut hasher);
            let hash = hasher.finish();

            let safe_53_bit = (hash & 0x001F_FFFF_FFFF_FFFF) as i64;
            if safe_53_bit == 0 { -1 } else { -safe_53_bit }
        });

    let artwork_url = tag.as_ref().and_then(|t| get_txxx(t, SC_ARTWORK_URL));
    let source_url = tag.as_ref().and_then(|t| get_txxx(t, SC_SOURCE_URL));

    let tag_artist = tag
        .as_ref()
        .and_then(|t| t.artist())
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let tag_title = tag
        .as_ref()
        .and_then(|t| t.title())
        .map(str::trim)
        .filter(|s| !s.is_empty());

    let (artist, title) = match (tag_artist, tag_title) {
        (Some(a), Some(t)) => (a.to_string(), t.to_string()),
        (Some(a), None) => {
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown");
            (a.to_string(), stem.to_string())
        }
        (None, Some(t)) => ("Unknown".to_string(), t.to_string()),
        (None, None) => {
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown");
            crate::utils::filename::parse_track_metadata(stem, "Unknown")
        }
    };

    Some(TrackMetadata {
        id,
        artist,
        title,
        artwork_url,
        source_url,
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_extract_track_metadata_without_tags() {
        let path1 = PathBuf::from("/var/lib/vortex-dl/Beauty in the Pain - Rebouz (192k).mp3");
        let path2 = PathBuf::from("/var/lib/vortex-dl/Beauty in the Pain - Rebouz (192k).mp3");
        let path3 = PathBuf::from("/var/lib/vortex-dl/Other Track.flac");

        let meta1 = extract_track_metadata(&path1).unwrap();
        let meta2 = extract_track_metadata(&path2).unwrap();
        let meta3 = extract_track_metadata(&path3).unwrap();

        assert_eq!(
            meta1.id, meta2.id,
            "Same path should yield same deterministic ID"
        );
        assert_ne!(
            meta1.id, meta3.id,
            "Different paths should yield different IDs"
        );
        assert!(
            meta1.id < 0,
            "Generated ID for local tracks should be negative"
        );
        assert_eq!(meta1.artist, "Beauty in the Pain");
        assert_eq!(meta1.title, "Rebouz (192k)");
    }
}
