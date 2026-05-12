use anyhow::Result;
use colored::Colorize;
use id3::{
    Content, Tag, TagLike, Version,
    frame::{ExtendedText, Frame, Picture, PictureType},
};

fn get_tag(path: &str) -> Result<Tag> {
    match Tag::read_from_path(path) {
        Ok(tag) => Ok(tag),
        Err(id3::Error {
            kind: id3::ErrorKind::NoTag,
            ..
        }) => Ok(Tag::new()),
        Err(e) => {
            eprintln!(
                "{} Failed to read ID3 tag at {}: {}. Starting fresh.",
                "[WARN]".yellow().bold(),
                path,
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
    pub path: &'a str,
    pub sc_id: &'a str,
    pub title: &'a str,
    pub artist: &'a str,
    pub artwork_url: Option<&'a str>,
    pub source_url: Option<&'a str>,
    pub artwork_data: Option<Vec<u8>>,
}

pub fn save_track_info(args: SaveTrackArgs) -> Result<()> {
    let mut tag = get_tag(args.path)?;

    tag.set_title(args.title);
    tag.set_artist(args.artist);

    set_txxx(&mut tag, crate::constants::SC_IDENTIFIER, args.sc_id);

    if let Some(url) = args.artwork_url {
        set_txxx(&mut tag, crate::constants::SC_ARTWORK_URL, url);
    }
    if let Some(url) = args.source_url {
        set_txxx(&mut tag, crate::constants::SC_SOURCE_URL, url);
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

pub fn read_custom_field(path: &str, key: &str) -> Option<String> {
    Tag::read_from_path(path).ok().and_then(|tag| {
        tag.extended_texts()
            .find(|f| f.description == key)
            .map(|f| f.value.clone())
    })
}
