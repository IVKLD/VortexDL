use id3::frame::{ExtendedText, Frame, Picture, PictureType};
use id3::{Content, Tag, TagLike, Version};

fn get_tag(path: &str) -> anyhow::Result<Tag> {
    match Tag::read_from_path(path) {
        Ok(tag) => Ok(tag),
        Err(id3::Error {
            kind: id3::ErrorKind::NoTag,
            ..
        }) => Ok(Tag::new()),
        Err(e) => Err(e.into()),
    }
}

pub fn save_track_info(
    path: &str,
    sc_id: &str,
    artwork_url: Option<&str>,
    artwork_data: Option<Vec<u8>>,
) -> anyhow::Result<()> {
    let mut tag = get_tag(path)?;

    let id_frame = Frame::with_content(
        "TXXX",
        Content::ExtendedText(ExtendedText {
            description: crate::models::SC_IDENTIFIER.to_string(),
            value: sc_id.to_string(),
        }),
    );
    tag.remove_extended_text(Some(crate::models::SC_IDENTIFIER), None);
    tag.add_frame(id_frame);

    if let Some(url) = artwork_url {
        let url_frame = Frame::with_content(
            "TXXX",
            Content::ExtendedText(ExtendedText {
                description: crate::models::SC_ARTWORK_URL.to_string(),
                value: url.to_string(),
            }),
        );
        tag.remove_extended_text(Some(crate::models::SC_ARTWORK_URL), None);
        tag.add_frame(url_frame);
    }

    if let Some(data) = artwork_data {
        let pic = Picture {
            mime_type: "image/jpeg".to_string(),
            picture_type: PictureType::CoverFront,
            description: "Cover".to_string(),
            data,
        };
        tag.remove_all_pictures();
        tag.add_frame(Frame::with_content("APIC", Content::Picture(pic)));
    }

    tag.write_to_path(path, Version::Id3v24)?;
    Ok(())
}

pub fn read_custom_field(path: &str, key: &str) -> Option<String> {
    Tag::read_from_path(path).ok().and_then(|tag| {
        tag.extended_texts()
            .find(|f| f.description == key)
            .map(|f| f.value.clone())
    })
}
