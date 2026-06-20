use std::path::Path;

use anyhow::Result;
use colored::Colorize;
use id3::{
    Content, Error as Id3Error, ErrorKind as Id3ErrorKind, Tag, TagLike, Version,
    frame::{ExtendedText, Frame, Picture, PictureType},
};

use crate::constants::{SC_ARTWORK_URL, SC_IDENTIFIER, SC_POSITION, SC_SOURCE_URL};

fn get_tag(path: impl AsRef<Path>) -> Result<Tag> {
    let path = path.as_ref();
    match Tag::read_from_path(path) {
        Ok(tag) => Ok(tag),
        Err(Id3Error {
            kind: Id3ErrorKind::NoTag,
            ..
        }) => Ok(Tag::new()),
        Err(e) => {
            eprintln!(
                "{} Failed to read ID3 tag at {}: {}. Starting fresh.",
                "[WARN]".yellow().bold(),
                path.display(),
                e
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
    pub artwork_url: Option<&'a str>,
    pub source_url: Option<&'a str>,
    pub position: Option<u32>,
    pub artwork_data: Option<Vec<u8>>,
}

pub fn save_track_info(args: SaveTrackArgs) -> Result<()> {
    let mut tag = get_tag(args.path)?;

    tag.set_title(args.title);
    tag.set_artist(args.artist);

    set_txxx(&mut tag, SC_IDENTIFIER, args.sc_id);

    if let Some(url) = args.artwork_url {
        set_txxx(&mut tag, SC_ARTWORK_URL, url);
    }
    if let Some(url) = args.source_url {
        set_txxx(&mut tag, SC_SOURCE_URL, url);
    }
    if let Some(pos) = args.position {
        set_txxx(&mut tag, SC_POSITION, &pos.to_string());
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

pub fn update_track_position(path: impl AsRef<Path>, position: Option<u32>) -> Result<()> {
    let path = path.as_ref();
    let mut tag = get_tag(path)?;
    if let Some(pos) = position {
        set_txxx(&mut tag, SC_POSITION, &pos.to_string());
    } else {
        tag.remove_extended_text(Some(SC_POSITION), None);
    }
    tag.write_to_path(path, Version::Id3v23)?;
    Ok(())
}

pub struct TrackMetadata {
    pub id: i64,
    pub artist: String,
    pub title: String,
    pub artwork_url: Option<String>,
    pub source_url: Option<String>,
    pub position: Option<u32>,
}

fn get_txxx(tag: &Tag, key: &str) -> Option<String> {
    tag.extended_texts()
        .find(|f| f.description == key)
        .map(|f| f.value.clone())
}

pub fn extract_track_metadata(path: impl AsRef<Path>) -> Option<TrackMetadata> {
    let tag = Tag::read_from_path(path).ok()?;

    let id = get_txxx(&tag, SC_IDENTIFIER)?.parse().ok()?;
    let artwork_url = get_txxx(&tag, SC_ARTWORK_URL);
    let source_url = get_txxx(&tag, SC_SOURCE_URL);
    let position = get_txxx(&tag, SC_POSITION).and_then(|v| v.parse().ok());

    let artist = tag.artist().unwrap_or("Unknown").to_string();
    let title = tag.title().unwrap_or("Unknown").to_string();

    Some(TrackMetadata {
        id,
        artist,
        title,
        artwork_url,
        source_url,
        position,
    })
}
